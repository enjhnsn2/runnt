use std::collections::VecDeque;

use ndarray::arr2;
use tempfile::NamedTempFile;

use crate::activation::ActivationType;
use crate::initialization::InitializationType;
use crate::nn::NN;
use crate::regularization::Regularization;

#[test]
/// Test xor nn against known outputs
fn test_xor() {
    //setup initial  weights
    let mut nn = NN::new(&[2, 2, 1])
        .with_learning_rate(0.5)
        .with_hidden_type(ActivationType::Sigmoid)
        .with_output_type(ActivationType::Sigmoid);

    nn.weights = [arr2(&[[0.15, 0.2], [0.25, 0.3]]), arr2(&[[0.4], [0.45]])].to_vec();
    nn.bias = [arr2(&[[0.35, 0.35]]), arr2(&[[0.6]])].to_vec();

    //check forward
    let vals = nn.forward(&[0., 0.]);
    assert_eq!(vals, [0.750002384]);

    //check first 4
    let inputs = [[0., 0.], [0., 1.], [1., 0.], [1., 1.]];
    let outputs = [[0.], [1.], [1.], [0.]];

    let known_errors = [0.2812518, 0.034677744, 0.033219155, 0.28904837];
    let known_biases = [
        //1
        [arr2(&[[0.35, 0.35]]), arr2(&[[0.6]])].to_vec(),
        //2
        [arr2(&[[0.343179762, 0.342327237]]), arr2(&[[0.529687762]])].to_vec(),
        //3
        [arr2(&[[0.3452806, 0.34468588]]), arr2(&[[0.555233]])].to_vec(),
        //4
        [arr2(&[[0.3474572, 0.34712338]]), arr2(&[[0.5798897]])].to_vec(),
    ]
    .to_vec();

    for i in 0..4 {
        //check error
        assert_eq!(nn.bias, known_biases[i]);
        println!("{i}: biases ok");
        let error = nn.calc_error(&nn.forward(&inputs[i]), &outputs[i]);
        nn.fit_one(&inputs[i], &outputs[i]);
        assert_eq!(error, known_errors[i]);
        println!("{i}: errors ok");
    }
}

#[test]
fn xor_sgd() {
    fastrand::seed(1);
    let mut nn = crate::nn::NN::new(&[2, 8, 1])
        .with_learning_rate(0.2)
        .with_hidden_type(ActivationType::Sigmoid)
        .with_output_type(ActivationType::Sigmoid);

    let mut inp_out = [
        ([0., 0.], [0.1]),
        ([0., 1.], [1.]),
        ([1., 0.], [1.]),
        ([1., 1.], [0.]),
    ];

    let mut completed_steps = vec![];

    //run a few times, and get average completion epoch
    for _ in 0..5 {
        let mut results: VecDeque<f32> = VecDeque::new();

        results.clear();
        nn.reset_weights(InitializationType::Random);
        for steps in 0..20_000 {
            fastrand::shuffle(&mut inp_out);
            nn.fit_one(&inp_out[0].0, &inp_out[0].1);
            let err = nn.forward_error(&inp_out[0].0, &inp_out[0].1);

            results.push_back(err);
            if results.len() > 100 {
                results.pop_front();
                if results.iter().sum::<f32>() / 100. < 0.02 {
                    completed_steps.push(steps);
                    break;
                }
            }
        }
    }

    let avg = completed_steps.iter().sum::<usize>() / completed_steps.len();

    println!("len:{} avg:{avg}", completed_steps.len());
    assert!(completed_steps.len() == 5);
    assert!(avg < 8000);
}

#[test]
fn xor_gd() {
    //do mini batch gradient descent.
    //we take a few at a time
    fastrand::seed(1);
    let mut nn = crate::nn::NN::new(&[2, 8, 1])
        .with_learning_rate(0.8)
        .with_hidden_type(ActivationType::Sigmoid)
        .with_output_type(ActivationType::Sigmoid);

    let mut inp_out = [
        (vec![0f32, 0.], vec![0.1]),
        (vec![0., 1.], vec![1.]),
        (vec![1., 0.], vec![1.]),
        (vec![1., 1.], vec![0.]),
    ];

    let mut completed_steps = vec![];

    //run a few times, and get average completion epoch
    for _ in 0..5 {
        let mut results: VecDeque<f32> = VecDeque::new();

        results.clear();
        nn.reset_weights(InitializationType::Random);
        for steps in 0..20_000 {
            fastrand::shuffle(&mut inp_out);
            let ins = vec![&inp_out[0].0, &inp_out[1].0];
            let outs = vec![&inp_out[0].1, &inp_out[1].1];

            nn.fit_batch(&ins, &outs);

            let err: f32 = nn.forward_errors(&ins, &outs);

            results.push_back(err / ins.len() as f32);
            if results.len() > 100 {
                results.pop_front();
                if results.iter().sum::<f32>() / 100. < 0.02 {
                    completed_steps.push(steps);
                    break;
                }
            }
        }
    }

    let avg = completed_steps.iter().sum::<usize>() / completed_steps.len();

    println!("len:{} avg:{avg}", completed_steps.len());

    assert!(completed_steps.len() == 5);
    assert!(avg < 2000);
}
#[test]
///use this in documentation
fn readme() {
    use crate::dataset::*;
    use crate::nn::*;
    //XOR

    let inputs = [[0., 0.], [0., 1.], [1., 0.], [1., 1.]];
    let outputs = [[0.], [1.], [1.], [0.]];

    let mut nn = NN::new(&[2, 8, 1])
        .with_learning_rate(0.2)
        .with_hidden_type(ActivationType::Tanh)
        .with_output_type(ActivationType::Linear);

    for i in 0..5000 {
        nn.fit_one(&inputs[i % 4], &outputs[i % 4]);
    }

    //iris
    #[cfg_attr(rustfmt, rustfmt_skip)] 
    {
    let set = Dataset::builder()
    .read_csv("examples/data/iris.csv")
    .add_input_columns(&[0, 1, 2, 3], Conversion::NormaliseMean)
    .add_target_columns(&[4], Conversion::OneHot)
    .allocate_to_test_data(0.4)
    .build();

    let mut net = NN::new(&[set.input_size(), 32, set.target_size()]).with_learning_rate(0.15);
    net.train(&set, 100, 8, 10, ReportMetric::CorrectClassification);
    }

    //run only if have diamonds dataset

    if !Path::new(r"/temp/diamonds.csv").exists() {
        return;
    }

    //run only on release
    if cfg!(release_assertions) {
        let set = Dataset::builder()
            .read_csv(r"/temp/diamonds.csv")
            .allocate_to_test_data(0.2)
            .add_input_columns(&[0, 4, 5, 7, 8, 9], Conversion::NormaliseMean)
            .add_input_columns(&[1, 2, 3], Conversion::OneHot)
            .add_target_columns(
                &[6],
                Conversion::Function(|f| f.parse::<f32>().unwrap_or_default() / 1_000.),
            )
            .build();

        let save_path = r"/temp/network.txt";
        let mut net = if std::path::PathBuf::from_str(save_path).unwrap().exists() {
            NN::load(save_path)
        } else {
            NN::new(&[set.input_size(), 32, set.target_size()])
        };

        //run for 100 epochs, with batch size 32 and report mse every 10 epochs
        net.train(&set, 100, 32, 10, ReportMetric::RSquared);
        net.save(save_path);
    }
}
#[test]
fn test_save_load() {
    let nn = NN::new(&[10, 100, 10])
        .with_learning_rate(0.5)
        .with_regularization(Regularization::L1L2(0.1, 0.2))
        .with_hidden_type(ActivationType::Tanh)
        .with_output_type(ActivationType::Relu);

    let input = [1., 2., 3., 4., 5., 6., 7., 8., 9., 10.];
    let result1 = nn.forward(&input);
    let temp = NamedTempFile::new().unwrap();
    let path = temp.path();

    //test looks the same
    let orig = nn.get_weights();
    let orig_shape = nn.get_shape();
    println!("shape:{orig_shape:?} weights:{orig:?}");
    nn.save(&path);
    let nn2 = NN::load(path);
    let new = nn2.get_weights();
    let new_shape = nn2.get_shape();
    println!("shape:{new_shape:?} weights:{new:?}");
    assert_eq!(orig_shape, new_shape);
    assert_eq!(orig, new);

    //test result the same
    let result2 = nn2.forward(&input);
    assert_eq!(result1, result2);
}

#[test]
fn test_get_set_weights() {
    //the result before and after should be the same if set with the same weights
    let mut nn = NN::new(&[12, 12, 12]);

    let test = [1., 2., 3., 4., 5., 6., 7., 8., 9., 10., 11., 12.];
    let res1 = nn.forward(&test);
    let old = nn.get_weights();
    nn.set_weights(&old);

    let res2 = nn.forward(&test);
    println!("before {res1:?} after: {res2:?}");

    assert_eq!(res1, res2);
}

#[test]
fn test_softmax_splits_even() {
    let nn = NN::new(&[1, 4])
        .with_initialization(InitializationType::Fixed(1.))
        .with_softmax_and_crossentropy();
    let test = arr2(&[[5.], [3.]]); //they should all have the same weight so 0.25
    let out = nn.internal_forward(&test);
    let out = out.last().unwrap();
    for row in out.rows() {
        assert_eq!(row.to_vec(), [0.25, 0.25, 0.25, 0.25]);
    }
}

#[test]
fn test_softmax_sums_to_one() {
    let nn = NN::new(&[1, 10, 8]).with_softmax_and_crossentropy();
    let test = arr2(&[[5.], [3.]]);
    let out = nn.internal_forward(&test);
    let out = out.last().unwrap();
    for row in out.rows() {
        let sum = row.sum();
        assert!(sum.abs() - 1.0 < 0.00001);
    }
}

//test against known weights,
// calculated weights manually, and with keras
#[test]
fn known_weights() {
    let mut nn = NN::new(&[2, 3, 2])
        .with_output_type(ActivationType::Sigmoid)
        .with_learning_rate(1.);
    nn.set_weights(&[
        0.992836, -0.225479, 0.020898, -0.837574, -0.109230, -0.945061, 0.754052, 0.919311,
        -0.678429, -0.929159, 0.010026, 0.588599, -0.214042, 0.169462, -0.667714, -0.352136,
        0.762778,
    ]);
    assert_eq!(
        nn.get_weights(),
        &[
            0.992836, -0.225479, 0.020898, -0.837574, -0.109230, -0.945061, 0.754052, 0.919311,
            -0.678429, -0.929159, 0.010026, 0.588599, -0.214042, 0.169462, -0.667714, -0.352136,
            0.762778
        ]
    );

    let orig_weights = nn.get_weights();

    assert_eq!(nn.forward(&[-0.539233, 0.728828]), [0.43365017, 0.6171831]);
    nn.fit_one(&[-0.539233, 0.728828], &[0., 0.]);

    let diff = orig_weights
        .iter()
        .zip(nn.get_weights())
        .map(|(a, b)| a - b)
        .collect::<Vec<_>>();
    assert_eq!(
        diff,
        [
            0.012651682,
            -0.0033963174,
            0.006875435,
            -0.017100036,
            0.0045904666,
            -0.009292841,
            -0.023462355,
            0.006298423,
            -0.012750387,
            0.04295206,
            0.05880838,
            0.0770424,
            0.105483666,
            0.021434084,
            0.029346764,
            0.10650349,
            0.14582068
        ]
    );
}
