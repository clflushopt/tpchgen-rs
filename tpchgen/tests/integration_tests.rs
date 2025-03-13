//! Consistence and conformance test suite that runs against Trino's TPCH
//! Java implementation.
use flate2::read::GzDecoder;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use tpchgen::generators::{
    CustomerGenerator, LineItemGenerator, NationGenerator, OrderGenerator, PartGenerator,
    PartSupplierGenerator, RegionGenerator, SupplierGenerator,
};

fn read_csv_gz<P: AsRef<Path>>(path: P) -> Vec<String> {
    let file = File::open(path).expect("Failed to open file");
    let gz = GzDecoder::new(file);
    let reader = BufReader::new(gz);
    reader
        .lines()
        .collect::<Result<_, _>>()
        .expect("Failed to read lines")
}

fn test_generator<T, G, F>(
    generator_fn: F,
    reference_path: &str,
    scale_factor: f64,
    transform_fn: impl Fn(T) -> String,
) where
    G: Iterator<Item = T>,
    F: FnOnce(f64) -> G,
{
    let mut dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set"));
    dir.push(reference_path);

    // Read reference data.
    let mut reference_data = read_csv_gz(dir);

    // Generate data using our own generators.
    let generator = generator_fn(scale_factor);
    let generated_data: Vec<String> = generator.map(transform_fn).collect();

    // Drop the header since the original files won't have them.
    reference_data.drain(0..1);

    // Compare that we have the same number of records.
    assert_eq!(
        reference_data.len(),
        generated_data.len(),
        "Number of records doesn't match for {}. Reference: {}, Generated: {}",
        reference_path,
        reference_data.len(),
        generated_data.len()
    );

    for (i, (reference, generated)) in reference_data.iter().zip(generated_data.iter()).enumerate()
    {
        assert_eq!(
            reference, generated,
            "Record {} doesn't match for {}.\nReference: {}\nGenerated: {}",
            i, reference_path, reference, generated
        );
    }
}

#[test]
fn test_nation_sf_0_001() {
    test_generator(
        |_| NationGenerator::new().iter(),
        "data/csv/sf-0.001/nation.csv.gz",
        0.001,
        |nation| {
            format!(
                "{},{},{},{}",
                nation.n_nationkey, nation.n_name, nation.n_regionkey, nation.n_comment
            )
        },
    );
}

#[test]
fn test_region_sf_0_001() {
    test_generator(
        |_| RegionGenerator::new().iter(),
        "data/csv/sf-0.001/region.csv.gz",
        0.001,
        |region| {
            format!(
                "{},{},{}",
                region.r_regionkey, region.r_name, region.r_comment
            )
        },
    );
}

#[test]
fn test_part_sf_0_001() {
    test_generator(
        |sf| PartGenerator::new(sf, 1, 1).iter(),
        "data/csv/sf-0.001/part.csv.gz",
        0.001,
        |part| {
            format!(
                "{},{},{},{},{},{},{},{:.2},{}",
                part.p_partkey,
                part.p_name,
                part.p_mfgr,
                part.p_brand,
                part.p_type,
                part.p_size,
                part.p_container,
                part.p_retailprice,
                part.p_comment
            )
        },
    );
}

#[test]
fn test_supplier_sf_0_001() {
    test_generator(
        |sf| SupplierGenerator::new(sf, 1, 1).iter(),
        "data/csv/sf-0.001/supplier.csv.gz",
        0.001,
        |supplier| {
            format!(
                "{},{},{},{},{},{:.2},{}",
                supplier.s_suppkey,
                supplier.s_name,
                supplier.s_address,
                supplier.s_nationkey,
                supplier.s_phone,
                supplier.s_acctbal,
                supplier.s_comment
            )
        },
    );
}

#[test]
fn test_partsupp_sf_0_001() {
    test_generator(
        |sf| PartSupplierGenerator::new(sf, 1, 1).iter(),
        "data/csv/sf-0.001/partsupp.csv.gz",
        0.001,
        |ps| {
            format!(
                "{},{},{},{:.2},{}",
                ps.ps_partkey, ps.ps_suppkey, ps.ps_availqty, ps.ps_supplycost, ps.ps_comment
            )
        },
    );
}

#[test]
fn test_customer_sf_0_001() {
    test_generator(
        |sf| CustomerGenerator::new(sf, 1, 1).iter(),
        "data/csv/sf-0.001/customer.csv.gz",
        0.001,
        |customer| {
            format!(
                "{},{},{},{},{},{:.2},{},{}",
                customer.c_custkey,
                customer.c_name,
                customer.c_address,
                customer.c_nationkey,
                customer.c_phone,
                customer.c_acctbal,
                customer.c_mktsegment,
                customer.c_comment
            )
        },
    );
}

#[test]
fn test_orders_sf_0_001() {
    test_generator(
        |sf| OrderGenerator::new(sf, 1, 1).iter(),
        "data/csv/sf-0.001/orders.csv.gz",
        0.001,
        |order| {
            format!(
                "{},{},{},{:.2},{},{},{},{},{}",
                order.o_orderkey,
                order.o_custkey,
                order.o_orderstatus,
                order.o_totalprice,
                order.o_orderdate,
                order.o_orderpriority,
                order.o_clerk,
                order.o_shippriority,
                order.o_comment
            )
        },
    );
}

#[test]
fn test_lineitem_sf_0_001() {
    test_generator(
        |sf| LineItemGenerator::new(sf, 1, 1).iter(),
        "data/csv/sf-0.001/lineitem.csv.gz",
        0.001,
        |item| {
            format!(
                "{},{},{},{},{:.2},{:.2},{:.2},{:.2},{},{},{},{},{},{},{},{}",
                item.l_orderkey,
                item.l_partkey,
                item.l_suppkey,
                item.l_linenumber,
                item.l_quantity,
                item.l_extendedprice,
                item.l_discount,
                item.l_tax,
                item.l_returnflag,
                item.l_linestatus,
                item.l_shipdate,
                item.l_commitdate,
                item.l_receiptdate,
                item.l_shipinstruct,
                item.l_shipmode,
                item.l_comment
            )
        },
    );
}

#[test]
fn test_nation_sf_0_01() {
    test_generator(
        |_| NationGenerator::new().iter(),
        "data/csv/sf-0.01/nation.csv.gz",
        0.01,
        |nation| {
            format!(
                "{},{},{},{}",
                nation.n_nationkey, nation.n_name, nation.n_regionkey, nation.n_comment
            )
        },
    );
}

#[test]
fn test_region_sf_0_01() {
    test_generator(
        |_| RegionGenerator::new().iter(),
        "data/csv/sf-0.01/region.csv.gz",
        0.01,
        |region| {
            format!(
                "{},{},{}",
                region.r_regionkey, region.r_name, region.r_comment
            )
        },
    );
}

#[test]
fn test_part_sf_0_01() {
    test_generator(
        |sf| PartGenerator::new(sf, 1, 1).iter(),
        "data/csv/sf-0.01/part.csv.gz",
        0.01,
        |part| {
            format!(
                "{},{},{},{},{},{},{},{:.2},{}",
                part.p_partkey,
                part.p_name,
                part.p_mfgr,
                part.p_brand,
                part.p_type,
                part.p_size,
                part.p_container,
                part.p_retailprice,
                part.p_comment
            )
        },
    );
}

#[test]
fn test_supplier_sf_0_01() {
    test_generator(
        |sf| SupplierGenerator::new(sf, 1, 1).iter(),
        "data/csv/sf-0.01/supplier.csv.gz",
        0.01,
        |supplier| {
            format!(
                "{},{},{},{},{},{:.2},{}",
                supplier.s_suppkey,
                supplier.s_name,
                supplier.s_address,
                supplier.s_nationkey,
                supplier.s_phone,
                supplier.s_acctbal,
                supplier.s_comment
            )
        },
    );
}

#[test]
fn test_partsupp_sf_0_01() {
    test_generator(
        |sf| PartSupplierGenerator::new(sf, 1, 1).iter(),
        "data/csv/sf-0.01/partsupp.csv.gz",
        0.01,
        |ps| {
            format!(
                "{},{},{},{:.2},{}",
                ps.ps_partkey, ps.ps_suppkey, ps.ps_availqty, ps.ps_supplycost, ps.ps_comment
            )
        },
    );
}

#[test]
fn test_customer_sf_0_01() {
    test_generator(
        |sf| CustomerGenerator::new(sf, 1, 1).iter(),
        "data/csv/sf-0.01/customer.csv.gz",
        0.01,
        |customer| {
            format!(
                "{},{},{},{},{},{:.2},{},{}",
                customer.c_custkey,
                customer.c_name,
                customer.c_address,
                customer.c_nationkey,
                customer.c_phone,
                customer.c_acctbal,
                customer.c_mktsegment,
                customer.c_comment
            )
        },
    );
}

#[test]
fn test_orders_sf_0_01() {
    test_generator(
        |sf| OrderGenerator::new(sf, 1, 1).iter(),
        "data/csv/sf-0.01/orders.csv.gz",
        0.01,
        |order| {
            format!(
                "{},{},{},{:.2},{},{},{},{},{}",
                order.o_orderkey,
                order.o_custkey,
                order.o_orderstatus,
                order.o_totalprice,
                order.o_orderdate,
                order.o_orderpriority,
                order.o_clerk,
                order.o_shippriority,
                order.o_comment
            )
        },
    );
}

#[test]
fn test_lineitem_sf_0_01() {
    test_generator(
        |sf| LineItemGenerator::new(sf, 1, 1).iter(),
        "data/csv/sf-0.01/lineitem.csv.gz",
        0.01,
        |item| {
            format!(
                "{},{},{},{},{},{:.2},{:.2},{:.2},{},{},{},{},{},{},{},{}",
                item.l_orderkey,
                item.l_partkey,
                item.l_suppkey,
                item.l_linenumber,
                item.l_quantity,
                item.l_extendedprice,
                item.l_discount,
                item.l_tax,
                item.l_returnflag,
                item.l_linestatus,
                item.l_shipdate,
                item.l_commitdate,
                item.l_receiptdate,
                item.l_shipinstruct,
                item.l_shipmode,
                item.l_comment
            )
        },
    );
}
