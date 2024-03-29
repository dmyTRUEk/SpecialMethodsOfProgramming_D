//! Very sophisticated data fitter.

#![feature(box_patterns)]

use std::{env, time::Instant};

use rand::{thread_rng, Rng};

mod extensions;
mod fit;
mod float_type;
mod function;
mod function_and_params;
mod param;
mod params;
mod point;
mod points;
mod utils_io;

use crate::{
    fit::{DiffFunctionType, FitAlgorithmType, FitResults, fit},
    float_type::float,
    function::Function,
    function_and_params::{FunctionAndParams, ToStringForPlot},
    params::Params,
    points::{ImplPoints, Points},
};


const CUSTOM_FUNCTION_FIT: bool = false;

mod fit_params {
    use super::*;
    pub const FIT_ALGORITHM_MIN_STEP: float = if CUSTOM_FUNCTION_FIT { 1e-4 } else { 1e-3 };
    pub const FIT_RESIDUE_EVALS_MAX: u32 = if CUSTOM_FUNCTION_FIT { 1_000_000 } else { 10_000 };
    pub const FIT_ALGORITHM_TYPE    : FitAlgorithmType = FitAlgorithmType::PatternSearch;
    pub const RESIDUAL_FUNCTION_TYPE: DiffFunctionType = DiffFunctionType::DySquared;
}

mod patter_search_params {
    use super::*;
    pub const INITIAL_STEP: float = 1.;
    pub const ALPHA: float = 2.;         // step increase coefficient
    pub const BETA : float = 1. / ALPHA; // step decrease coefficient
}

mod downhill_simplex_params {
    use super::*;
    pub const INITIAL_SIMPLEX_SCALE: float = 1.;
    pub const PARAMS_DIFF_TYPE: DiffFunctionType = DiffFunctionType::DyAbs;
}

// TODO(refactor): make private (non `pub`).
pub const FUNCTION_PARAM_VALUE_MIN: float = -9.;
pub const FUNCTION_PARAM_VALUE_MAX: float =  9.;

const FUNCTION_PARAMS_AMOUNT_MIN: usize = 1;
const FUNCTION_PARAMS_AMOUNT_MAX: usize = 7;

const FUNCTION_COMPLEXITY_MIN: u32 = 5;
const FUNCTION_COMPLEXITY_MAX: u32 = 30;

const FIT_RESIDUE_THRESHOLD: float = float::INFINITY;


fn main() {
    let args: Vec<_> = env::args().collect();
    let filename: &str = match &args[..] {
        [_, filename] => filename,
        [_] => panic!("Filename not provided."),
        [] => panic!("Unexpected CLI args number."),
        _ => panic!("Too many CLI args.")
    };

    // fit_custom(filename);
    // return;

    // benchmark_fit();
    // return;

    let points = Points::load_from_file(filename);

    let mut rng = thread_rng();
    let mut best_f_and_res: (FunctionAndParams, float) = (FunctionAndParams::gen_random_params_from_function(Function::X), FIT_RESIDUE_THRESHOLD);
    let mut funcs_generated: u64 = 0;
    let mut funcs_fitted: u64 = 0;
    // let mut fit_residue_evals_array = [0_u32; 100];
    // let mut i: usize = 0;
    let time_begin = Instant::now();
    loop {
        // if funcs_fitted >= 50_000 {
        //     print_stats(funcs_generated, funcs_fitted, time_begin);
        //     // println!("fit_residue_evals_array = {:?}", fit_residue_evals_array);
        //     // println!("fit_residue_evals_array.total = {}", fit_residue_evals_array.iter().sum::<u32>());
        //     return
        // }
        // if funcs_fitted % 100 == 0 {
        //     let time_now = Instant::now();
        //     let time_delta = time_now - time_begin;
        //     if time_delta.as_secs() >= 5*60 {
        //         print_stats(funcs_generated, funcs_fitted, time_begin);
        //         return
        //     }
        // }
        funcs_generated += 1;

        let mut f = if CUSTOM_FUNCTION_FIT {
            FunctionAndParams::gen_random_params_from_function_and_some_params(
                Function::from_str("(tan(a - (exp((b*x+i) * (c - sin(x))))^(d - cos(cos(h*x+g)))))^2").unwrap(),
                // a b c d g h i
                Params::from_array([
                    ('a', -8.0897832478892900),
                    ('b',  0.1577825221898550),
                    ('c', -1.8801382154778565),
                    ('d',  1.2566549816649228),
                    ('g', -1.2261250730153570),
                    ('h', -1.3636558727828500),
                    ('i',  0.6369819759696052),
                ])
            )
        } else {
            let complexity: u32 = rng.gen_range(FUNCTION_COMPLEXITY_MIN ..= FUNCTION_COMPLEXITY_MAX);
            let mut f = FunctionAndParams::gen_random_function_and_params(complexity);
            // println!("f = {}", f.to_string());
            f = f.simplify();
            // println!("f = {}", f.to_string());
            f
        };

        if !(FUNCTION_PARAMS_AMOUNT_MIN ..= FUNCTION_PARAMS_AMOUNT_MAX).contains(&f.params.amount()) {
            if CUSTOM_FUNCTION_FIT { panic!("not enough or too many params in function") }
            // println!("too many params in generated function, skipping");
            continue;
        }
        // println!("f = {}", f.to_string());
        // println!("fitting...");
        let fit_results = fit(&mut f, &points);
        // println!("fit_residue = {:?}", fit_results);
        // press_enter_to_continue();
        if fit_results.is_none() { continue }
        let FitResults { fit_residue, fit_residue_evals } = fit_results.unwrap();
        funcs_fitted += 1;
        // fit_residue_evals_array[i] = fit_residue_evals;
        if !fit_residue.clone().is_finite() { continue }

        if fit_residue <= best_f_and_res.1 {
            print_stats(funcs_generated, funcs_fitted, time_begin);
            println!();
            println!("FOUND NEW BEST FUNCTION:");
            println!("fit_residue_evals: {}", fit_residue_evals);
            println!("FUNCTION:");
            println!("{}", f.to_string_for_plot());
            println!("\"residue = {}", fit_residue);
            best_f_and_res = (f, fit_residue);
            println!("{}", "-".repeat(42));
            // wait_for_enter();
            println!();
            println!("searching...");
        }
        // i += 1;
    }
}


#[allow(dead_code)]
fn fit_custom(filename: &str) {
    let points = Points::load_from_file(filename);

    let mut f = match filename {
        "./data/fit_Dm_1.dat" => FunctionAndParams::new(
            Function::from_str("h + a*exp(k*(x-m))").unwrap(),
            Params::from_array([
                ('h', 0.0),
                ('a', 1.0), ('k', -1.0), ('m', 0.0),
            ])
        ),
        "./data/fit_Dm_1c.dat" => todo!(),
        "./data/fit_Dm_2.dat"  => todo!(),
        "./data/fit_Dm_2c.dat" => todo!(),
        "./data/fit_Dm_3.dat"  => FunctionAndParams::new(
            Function::from_str("h + a*sin(k*(x-m))").unwrap(),
            Params::from_array([
                ('h', 16.5),
                ('a', 13.0), ('k', 1.0), ('m', -0.5),
            ])
        ),
        "./data/fit_Dm_3c.dat" => todo!(),
        "./data/fit_Dm_4.dat"  => FunctionAndParams::new(
            Function::from_str("h + a*exp(-((x-m)/s)^2) + b*exp(-((x-n)/t)^2)").unwrap(),
            Params::from_array([
                ('h', 0.5),
                ('a', 5.0), ('m', 1.5), ('s', 0.6),
                ('b', 2.5), ('n', 3.5), ('t', 0.6),
            ])
        ),
        _ => todo!()
    };

    println!("f = {}", f.to_string());
    // f = f.simplify();
    // println!("f = {}", f.to_string());
    let fit_results = fit(&mut f, &points);
    let (fit_residue, _f_evals) = match fit_results {
        Some(FitResults { fit_residue, fit_residue_evals }) => {
            println!("fit_residue_evals: {}", fit_residue_evals);
            (fit_residue, fit_residue_evals)
        }
        None => {
            // println!("Unable to fit: {}", e);
            println!("Unable to fit");
            return;
        }
    };
    println!("FUNCTION:");
    println!("{}", f.to_string_for_plot());
    println!("\"residue = {}", fit_residue);
    println!("{}", "-".repeat(42));
}


fn print_stats(funcs_generated: u64, funcs_fitted: u64, time_begin: Instant) {
    let time_now = Instant::now();
    let time_delta = time_now - time_begin;
    let millis_passed = time_delta.as_micros();
    let funcs_generated_per_sec = 1e6 * (funcs_generated as float) / (millis_passed as float);
    let funcs_fitted_per_sec    = 1e6 * (funcs_fitted    as float) / (millis_passed as float);
    fn number_to_decimal_places(x: float) -> usize {
        match x {
            x if x > 1000. => 0,
            x if x > 100.0 => 1,
            x if x > 10.00 => 2,
            x if x > 1.000 => 3,
            x if x > 0.100 => 3,
            x if x > 0.010 => 4,
            x if x > 0.001 => 5,
            _              => 6
        }
    }
    fn format(x: float) -> String {
        // format_with_decimal_places(x, number_to_decimal_places(x))
        format!("{0:.1$}", x, number_to_decimal_places(x))
    }
    println!("funcs generated: {}\t{}/s", funcs_generated, format(funcs_generated_per_sec));
    println!("funcs fitted   : {}\t{}/s", funcs_fitted, format(funcs_fitted_per_sec));
}



#[allow(dead_code)]
fn benchmark_fit(filename: &str) {
    let points = Points::load_from_file(filename);
    // println!("{:#?}", points);

    let params = Params::from_array([
        ('f', -1.),
        ('q', -1.),
        ('w', -1.),
    ]);
    let mut f = FunctionAndParams::new(
        Function::from_str("((exp(x) / x)^(w))^(q) * (x * f)").unwrap(),
        // Function::Mul {
        //     lhs: box Function::Pow {
        //         lhs: box Function::Pow {
        //             lhs: box Function::Div {
        //                 lhs: box Function::Exp {
        //                     value: box Function::X
        //                 },
        //                 rhs: box Function::X,
        //             },
        //             rhs: box Function::Param { name: 'w' }
        //         },
        //         rhs: box Function::Param { name: 'q' }
        //     },
        //     rhs: box Function::Mul {
        //         lhs: box Function::X,
        //         rhs: box Function::Param { name: 'f' }
        //     }
        // },
        params.clone()
    );
    let time_begin = Instant::now();
    for _ in 0..10 {
        f.params = params.clone();
        // println!("f = {}", f.to_string());
        let fit_results = fit(&mut f, &points);
        println!("fit_residue = {:?}", fit_results);
    }
    let time_end = Instant::now();
    println!("finished in {} ms.", (time_end - time_begin).as_millis());
}

