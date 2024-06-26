use std::{
    io::{BufReader, Read},
    path::{Path, PathBuf},
    time::Instant,
};

use runnt::{
    activation::ActivationType,
    nn::{max_index_equal, NN},
};
//Classification example
pub fn main() {
    fastrand::seed(1);

    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        println!(
            r"
Files can be downloaded from http://yann.lecun.com/exdb/mnist/
Extract the files
Pass in path of the 4 mnist files which have been extracted
cargo run --release --example mnist -- /tmp/mnist
        "
        );
        return;
    }

    let mut nn = NN::new(&[784, 128, 10])
        .with_hidden_type(ActivationType::Sigmoid)
        .with_output_type(ActivationType::Sigmoid)
        .with_learning_rate(0.15);

    let path = &args[1];
    let (mut training, test) = get_train_test(path);

    let start = Instant::now();

    for epoch in 1..20 {
        fastrand::shuffle(&mut training);

        let inputs = training.iter().map(|x| &x.0).collect::<Vec<_>>();
        let targets = training.iter().map(|x| &x.1).collect::<Vec<_>>();
        nn.fit(&inputs, &targets, 10);
        let (test_acc, test_mse) = get_acc_mse(&nn, &test);
        let (train_acc, train_mse) = get_acc_mse(&nn, &training);
        println!(
            "epoch {epoch} train mse:{} test mse:{} train acc:{}% test acc:{}% in {:.2}s",
            train_mse,
            test_mse,
            train_acc * 100.,
            test_acc * 100.,
            start.elapsed().as_secs_f32()
        );
    }
}

fn get_acc_mse(nn: &NN, data: &Vec<(Vec<f32>, Vec<f32>)>) -> (f32, f32) {
    let mut sum = 0;
    let mut sse = 0.;
    for i in 0..data.len() {
        let pred = nn.forward(data[i].0.as_slice());

        if max_index_equal(&pred, &data[i].1) {
            sum += 1
        }
        sse += nn.calc_error(&pred, &data[i].1);
    }

    let mse = sse / (data.len() as f32);
    let mean_acc = (sum as f32) / (data.len() as f32);
    (mean_acc, mse)
}

fn get_train_test(
    path: impl AsRef<Path>,
) -> (Vec<(Vec<f32>, Vec<f32>)>, Vec<(Vec<f32>, Vec<f32>)>) {
    // data can be downloaded from http://yann.lecun.com/exdb/mnist/
    // 2 train files, one for labels, one for images
    // 2 test files

    // files are:
    // TRAIN_IMAGE "train-images.idx3-ubyte"
    // TRAIN_LABEL "train-labels.idx1-ubyte"
    // TEST_IMAGE:"t10k-images.idx3-ubyte"
    // TEST_LABEL:"t10k-labels.idx1-ubyte"
    // sometimes there are . instead of -, or vice versa, so we remove

    let mut label_path = PathBuf::from("");
    let mut image_path = PathBuf::from("");
    let mut test_label_path = PathBuf::from("");
    let mut test_image_path = PathBuf::from("");
    std::fs::read_dir(path)
        .expect("Could not find mnist folder")
        .for_each(|f| {
            if let Ok(ref fi) = f {
                match fi
                    .file_name()
                    .to_ascii_lowercase()
                    .to_str()
                    .unwrap_or_default()
                    .replace(['-', '.'], "")
                    .as_str()
                {
                    "trainimagesidx3ubyte" => {
                        image_path = f.unwrap().path();
                    }
                    "trainlabelsidx1ubyte" => {
                        label_path = f.unwrap().path();
                    }
                    "t10kimagesidx3ubyte" => {
                        test_image_path = f.unwrap().path();
                    }
                    "t10klabelsidx1ubyte" => {
                        test_label_path = f.unwrap().path();
                    }
                    _ => {}
                }
            }
        });

    println!("getting data");

    let train = load_image_label(image_path, label_path);
    let test = load_image_label(test_image_path, test_label_path);
    (train, test)
}

fn load_image_label(
    image_path: std::path::PathBuf,
    label_path: std::path::PathBuf,
) -> Vec<(Vec<f32>, Vec<f32>)> {
    //labels
    let mut br =
        BufReader::new(std::fs::File::open(label_path).expect("Could not find mnist file"));
    let mut label_buf = Vec::new();
    br.read_to_end(&mut label_buf)
        .expect("Could not read labels");

    let label_result: Vec<Vec<f32>> = label_buf
        .into_iter()
        .skip(8) //magic number + count
        .map(|a| {
            let mut label = vec![0f32, 0., 0., 0., 0., 0., 0., 0., 0., 0.];
            label[a as usize] = 1.;
            label
        })
        .collect();

    //data
    let mut br =
        BufReader::new(std::fs::File::open(image_path).expect("Could not find mnist file"));
    let mut data_buf = Vec::new();
    br.read_to_end(&mut data_buf)
        .expect("Could not read images");

    let data_result: Vec<Vec<f32>> = data_buf
        .into_iter()
        .skip(16)
        .collect::<Vec<u8>>()
        .as_slice()
        .chunks(28 * 28)
        .map(|a| a.iter().map(|&b| ((b as usize) as f32) / 255.0).collect())
        .collect();

    println!(
        "got {} labels {} images",
        label_result.len(),
        data_result.len()
    );

    data_result.into_iter().zip(label_result).collect()
}
