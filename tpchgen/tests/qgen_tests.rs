//! Conformance tests for the query parameter substitution against `qgen`
//! from TPC-H 2.17.
use std::path::PathBuf;
use tpchgen::q_and_a::qgen::{stream_order, QueryGenerator, Substitution};

/// Compares the parameters of all queries against `data/qgen/parameters.tsv`,
/// which holds the parameters `qgen -r <seed> -s <sf> -l <log>` logs.
#[test]
fn test_seeded_parameters_match_qgen() {
    let mut path =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set"));
    path.push("data/qgen/parameters.tsv");
    let reference = std::fs::read_to_string(path).unwrap();

    let mut count = 0;
    for line in reference.lines().filter(|line| !line.starts_with('#')) {
        let mut fields = line.split('\t');
        let seed: u32 = fields.next().unwrap().parse().unwrap();
        let scale_factor: f64 = fields.next().unwrap().parse().unwrap();
        let query: i32 = fields.next().unwrap().parse().unwrap();
        let expected: Vec<&str> = fields.collect();

        let generator = QueryGenerator::new(scale_factor, Substitution::Seeded(seed));
        assert_eq!(
            generator.parameters(query).unwrap(),
            expected,
            "Q{query} with seed {seed} at scale factor {scale_factor}"
        );
        count += 1;
    }
    assert_eq!(count, 25 * 22);
}

#[test]
fn test_validation_parameters() {
    let generator = QueryGenerator::new(1.0, Substitution::Validation);
    assert_eq!(generator.parameters(1).unwrap(), ["90"]);
    assert_eq!(
        generator.parameters(11).unwrap(),
        ["GERMANY", "0.0001000000"]
    );
    assert_eq!(
        generator.parameters(22).unwrap(),
        ["13", "31", "23", "29", "30", "18", "17"]
    );

    // the Q11 fraction is scaled in both modes
    let generator = QueryGenerator::new(100.0, Substitution::Validation);
    assert_eq!(
        generator.parameters(11).unwrap(),
        ["GERMANY", "0.0000010000"]
    );
}

#[test]
fn test_invalid_query_numbers() {
    let generator = QueryGenerator::new(1.0, Substitution::Validation);
    assert!(generator.parameters(0).is_none());
    assert!(generator.query(23, 0).is_none());
}

#[test]
#[should_panic(expected = "invalid qgen seed")]
fn test_degenerate_seed() {
    QueryGenerator::new(1.0, Substitution::Seeded(0));
}

#[test]
fn test_validation_queries() {
    let generator = QueryGenerator::new(1.0, Substitution::Validation);

    let q1 = generator.query(1, 0).unwrap();
    assert!(q1.starts_with("select\n\tl_returnflag,"), "{q1}");
    assert!(
        q1.contains("date '1998-12-01' - interval '90' day\n"),
        "{q1}"
    );
    assert!(q1.ends_with("\tl_linestatus;\n"), "{q1}");

    let q2 = generator.query(2, 0).unwrap();
    assert!(q2.contains("\tand p_size = 15\n\tand p_type like '%BRASS'\n"));
    assert!(q2.ends_with("\tp_partkey\nlimit 100;\n"), "{q2}");

    let q16 = generator.query(16, 0).unwrap();
    assert!(
        q16.contains("p_size in (49, 14, 23, 45, 19, 3, 36, 9)"),
        "{q16}"
    );

    // no directives or comments left in any query
    for query in 1..=22 {
        let sql = generator.query(query, 0).unwrap();
        assert!(!sql.contains(':'), "Q{query}: {sql}");
        assert!(!sql.contains("--"), "Q{query}: {sql}");
        let limits = sql.matches("limit").count();
        let expected = usize::from([2, 3, 10, 18, 21].contains(&query));
        assert_eq!(limits, expected, "Q{query}: {sql}");
    }
}

#[test]
fn test_stream_queries() {
    let generator = QueryGenerator::new(1.0, Substitution::Seeded(101));

    let q15 = generator.query(15, 7).unwrap();
    assert!(q15.starts_with("create view revenue7 (supplier_no, total_revenue) as\n"));
    assert!(q15.contains("\n\trevenue7\nwhere\n"), "{q15}");
    assert!(q15.ends_with("\ndrop view revenue7;\n"), "{q15}");

    assert_eq!(
        stream_order(0),
        [14, 2, 9, 20, 6, 17, 18, 8, 21, 13, 3, 22, 16, 4, 11, 15, 1, 10, 19, 5, 7, 12]
    );
    // the permutations repeat after 41 streams
    assert_eq!(stream_order(41), stream_order(0));

    let stream = generator.stream(2);
    let order: Vec<i32> = stream.iter().map(|(query, _)| *query).collect();
    assert_eq!(order, stream_order(2));
    for (query, sql) in stream {
        assert_eq!(sql, generator.query(query, 2).unwrap());
    }
}
