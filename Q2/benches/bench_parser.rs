

use parser_sample::Parser;
use parser_sample::parser::ParseError;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn parse_single_once(data: &String) {
    let mut parser = Parser::new(data);

    match parser.parse_single() {
        Err(error) => {
            assert!(false, "Error parsing 100 entries: {}", error);
        },
        Ok(_) => {},
    }
}

fn parse_single_100_times(data: &String) {
    let mut parser = Parser::new(data);

    for _ in 0..100 {
        match parser.parse_single() {
            Err(error) => {
                assert!(false, "Error parsing 100 entries: {}", error);
            },
            Ok(_) => {},
        }
    }
}

fn parse_entire_data(data: &String) {
    let mut parser = Parser::new(data);

    let mut count = 0;
    loop {
        match parser.parse_single() {
            Err(ParseError::EndOfData) => break,
            Err(error) => {
                assert!(false, "parse_single produced a non-EndOfData error: {}", error);
            }
            Ok(_) => count+=1,
        }
    }

    assert_eq!(count, 1436);
}

fn parser_benchmark(criterion: &mut Criterion) {
    // Loading the data is not part of what we want to measure
    let file_path = "./assets/body_text.json";
    let file = match std::fs::read_to_string(file_path) {
        Ok(file) => file,
        Err(error) => {
            assert!(false, "Reading the asset file failed: {}", error);
            return;
        }
    };

    criterion.bench_function("single parse_single() calls", |bencher| {
        bencher.iter(|| parse_single_once(&file));
    });

    criterion.bench_function("100 parse_single() calls", |bencher| {
        bencher.iter(|| parse_single_100_times(&file));
    });

    criterion.bench_function("parsing entire data", |bencher| {
        bencher.iter(|| parse_entire_data(&file));
    });
}

criterion_group!(benches, parser_benchmark);

criterion_main!(benches);
