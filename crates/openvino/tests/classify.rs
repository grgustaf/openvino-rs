//! Demonstrates using `openvino-rs` to classify an image. This relies on the test fixtures
//! prepared in the `fixture` directory. See the [README](fixture/README.md) for details on how
//! the fixture was prepared.
mod fixture;

use fixture::Fixture;
use opencv;
use opencv::core::{MatTrait, MatTraitManual};
use openvino::{Blob, Core, Layout, Precision, ResizeAlgorithm, TensorDesc};

#[test]
fn classification() {
    let mut core = Core::new(None).unwrap();
    let mut network = core
        .read_network_from_file(
            &Fixture::graph().to_string_lossy(),
            &Fixture::weights().to_string_lossy(),
        )
        .unwrap();

    let input_name = &network.get_input_name(0).unwrap();
    assert_eq!(input_name, "image_tensor");
    let output_name = &network.get_output_name(0).unwrap();
    assert_eq!(output_name, "DetectionOutput");

    // Prepare inputs and outputs.
    network
        .set_input_resize_algorithm(input_name, ResizeAlgorithm::RESIZE_BILINEAR)
        .unwrap();
    network.set_input_layout(input_name, Layout::NHWC).unwrap();
    network
        .set_input_precision(input_name, Precision::U8)
        .unwrap();
    network
        .set_output_precision(output_name, Precision::FP32)
        .unwrap();

    // Load the network.
    let mut executable_network = core.load_network(&network, "CPU").unwrap();
    let mut infer_request = executable_network.create_infer_request().unwrap();
    // TODO eventually, this should not panic: infer_request.set_batch_size(1).unwrap();

    // Read the image.
    let mat = opencv::imgcodecs::imread(
        &*Fixture::image().to_string_lossy(),
        opencv::imgcodecs::IMREAD_COLOR,
    )
    .unwrap();
    let desc = TensorDesc::new(
        Layout::NHWC,
        &[
            1,
            mat.channels().unwrap() as u64,
            mat.size().unwrap().height as u64,
            mat.size().unwrap().width as u64, // TODO .try_into().unwrap()
        ], // {1, (size_t)img.mat_channels, (size_t)img.mat_height, (size_t)img.mat_width}
        Precision::U8,
    );

    // Extract the OpenCV mat bytes and place them in an OpenVINO blob.
    let data = unsafe { std::slice::from_raw_parts(mat.data().unwrap() as *const u8, desc.len()) };
    let blob = Blob::new(desc, data).unwrap();

    // Execute inference.
    infer_request.set_blob(input_name, blob).unwrap();
    infer_request.infer().unwrap();
    let mut results = infer_request.get_blob(output_name).unwrap();
    let buffer = results.buffer::<f32>().unwrap().to_vec();

    // Sort results.
    let mut results: Results = buffer
        .iter()
        .enumerate()
        .map(|(c, p)| Result(c, *p))
        .collect();
    results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    assert_eq!(
        &results[..5],
        &[
            Result(15, 59.0),
            Result(1, 1.0),
            Result(8, 1.0),
            Result(12, 1.0),
            Result(16, 0.9939936),
        ][..]
    )
}

/// A structure for holding the `(category, probability)` pair extracted from the output tensor of
/// the OpenVINO classification.
#[derive(Debug, PartialEq)]
struct Result(usize, f32);
type Results = Vec<Result>;