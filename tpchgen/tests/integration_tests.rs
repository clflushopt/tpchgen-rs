//! Consistence and conformance test suite that runs against Trino's TPCH
//! Java implementation.
use flate2::read::GzDecoder;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use tpchgen::generators::{
    CustomerGenerator, DeleteGenerator, LineItemGenerator, NationGenerator, OrderGenerator,
    PartGenerator, PartSuppGenerator, RegionGenerator, SupplierGenerator, UpdateSet,
};

fn read_tbl_gz<P: AsRef<Path>>(path: P) -> Vec<String> {
    let file = File::open(path).expect("Failed to open file");
    let gz = GzDecoder::new(file);
    let reader = BufReader::new(gz);
    reader
        .lines()
        .collect::<Result<_, _>>()
        .expect("Failed to read lines")
}

fn test_generator<T, I>(iter: I, reference_path: &str, transform_fn: impl Fn(T) -> String)
where
    I: Iterator<Item = T>,
{
    let mut dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set"));
    dir.push(reference_path);

    // Read reference data.
    let reference_data = read_tbl_gz(dir);

    // Generate data using our own generators.
    let generated_data: Vec<String> = iter.map(transform_fn).collect();

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
    let _sf = 0.001;
    let generator = NationGenerator::default();
    test_generator(generator.iter(), "data/sf-0.001/nation.tbl.gz", |nation| {
        nation.to_string()
    });
}

#[test]
fn test_region_sf_0_001() {
    let _sf = 0.001;
    let generator = RegionGenerator::default();
    test_generator(generator.iter(), "data/sf-0.001/region.tbl.gz", |region| {
        region.to_string()
    });
}

#[test]
fn test_part_sf_0_001() {
    let sf = 0.001;
    let generator = PartGenerator::new(sf, 1, 1);
    test_generator(generator.iter(), "data/sf-0.001/part.tbl.gz", |part| {
        part.to_string()
    });
}

#[test]
fn test_supplier_sf_0_001() {
    let sf = 0.001;
    let generator = SupplierGenerator::new(sf, 1, 1);
    test_generator(
        generator.iter(),
        "data/sf-0.001/supplier.tbl.gz",
        |supplier| supplier.to_string(),
    );
}

#[test]
fn test_partsupp_sf_0_001() {
    let sf = 0.001;
    let generator = PartSuppGenerator::new(sf, 1, 1);
    test_generator(generator.iter(), "data/sf-0.001/partsupp.tbl.gz", |ps| {
        ps.to_string()
    });
}

#[test]
fn test_customer_sf_0_001() {
    let sf = 0.001;
    let generator = CustomerGenerator::new(sf, 1, 1);
    test_generator(
        generator.iter(),
        "data/sf-0.001/customer.tbl.gz",
        |customer| customer.to_string(),
    );
}

#[test]
fn test_orders_sf_0_001() {
    let sf = 0.001;
    let generator = OrderGenerator::new(sf, 1, 1);
    test_generator(generator.iter(), "data/sf-0.001/orders.tbl.gz", |order| {
        order.to_string()
    });
}

#[test]
fn test_lineitem_sf_0_001() {
    let sf = 0.001;
    let generator = LineItemGenerator::new(sf, 1, 1);
    test_generator(generator.iter(), "data/sf-0.001/lineitem.tbl.gz", |item| {
        item.to_string()
    });
}

#[test]
fn test_nation_sf_0_01() {
    let sf = 0.01;
    let generator = NationGenerator::new(sf, 1, 1);
    test_generator(generator.iter(), "data/sf-0.01/nation.tbl.gz", |nation| {
        nation.to_string()
    });
}

#[test]
fn test_region_sf_0_01() {
    let sf = 0.01;
    let generator = RegionGenerator::new(sf, 1, 1);
    test_generator(generator.iter(), "data/sf-0.01/region.tbl.gz", |region| {
        region.to_string()
    });
}

#[test]
fn test_part_sf_0_01() {
    let sf = 0.01;
    let generator = PartGenerator::new(sf, 1, 1);
    test_generator(generator.iter(), "data/sf-0.01/part.tbl.gz", |part| {
        part.to_string()
    });
}

#[test]
fn test_supplier_sf_0_01() {
    let sf = 0.01;
    let generator = SupplierGenerator::new(sf, 1, 1);
    test_generator(
        generator.iter(),
        "data/sf-0.01/supplier.tbl.gz",
        |supplier| supplier.to_string(),
    );
}

#[test]
fn test_partsupp_sf_0_01() {
    let sf = 0.01;
    let generator = PartSuppGenerator::new(sf, 1, 1);
    test_generator(generator.iter(), "data/sf-0.01/partsupp.tbl.gz", |ps| {
        ps.to_string()
    });
}

#[test]
fn test_customer_sf_0_01() {
    let sf = 0.01;
    let generator = CustomerGenerator::new(sf, 1, 1);
    test_generator(
        generator.iter(),
        "data/sf-0.01/customer.tbl.gz",
        |customer| customer.to_string(),
    );
}

#[test]
fn test_orders_sf_0_01() {
    let sf = 0.01;
    let generator = OrderGenerator::new(sf, 1, 1);
    test_generator(generator.iter(), "data/sf-0.01/orders.tbl.gz", |order| {
        order.to_string()
    })
}

#[test]
fn test_lineitem_sf_0_01() {
    let sf = 0.01;
    let generator = LineItemGenerator::new(sf, 1, 1);
    test_generator(generator.iter(), "data/sf-0.01/lineitem.tbl.gz", |item| {
        item.to_string()
    })
}

/// Compares update set `update_set` (`dbgen -s <sf> -U <update_set>`)
fn test_update_set(sf: f64, update_set: i32, reference_dir: &str) {
    test_generator(
        OrderGenerator::update_set(sf, update_set, 1, 1).iter(),
        &format!("{reference_dir}/orders.tbl.u{update_set}.gz"),
        |order| order.to_string(),
    );
    test_generator(
        LineItemGenerator::update_set(sf, update_set, 1, 1).iter(),
        &format!("{reference_dir}/lineitem.tbl.u{update_set}.gz"),
        |item| item.to_string(),
    );
    test_generator(
        DeleteGenerator::new(sf, update_set).iter(),
        &format!("{reference_dir}/delete.{update_set}.gz"),
        |delete| delete.to_string(),
    );
}

#[test]
fn test_update_sets_sf_0_1() {
    test_update_set(0.1, 1, "data/updates/sf-0.1");
    test_update_set(0.1, 2, "data/updates/sf-0.1");
}

/// Update sets from 1000 on use the next sparse key slot
#[test]
fn test_update_set_1001_sf_0_01() {
    test_update_set(0.01, 1001, "data/updates/sf-0.01");
}

/// The parts of an update set concatenate to the whole update set
#[test]
fn test_update_set_parts() {
    let sf = 0.1;
    for update_set in [1, 1001] {
        let orders: Vec<_> = (1..=3)
            .flat_map(|part| OrderGenerator::update_set(sf, update_set, part, 3).iter())
            .map(|order| order.to_string())
            .collect();
        let whole: Vec<_> = OrderGenerator::update_set(sf, update_set, 1, 1)
            .iter()
            .map(|order| order.to_string())
            .collect();
        assert_eq!(orders, whole);

        let items: Vec<_> = (1..=3)
            .flat_map(|part| LineItemGenerator::update_set(sf, update_set, part, 3).iter())
            .map(|item| item.to_string())
            .collect();
        let whole: Vec<_> = LineItemGenerator::update_set(sf, update_set, 1, 1)
            .iter()
            .map(|item| item.to_string())
            .collect();
        assert_eq!(items, whole);
    }
}

#[test]
fn test_update_set_row_count() {
    assert_eq!(UpdateSet::row_count(1.0), 1500);
    assert_eq!(UpdateSet::row_count(100.0), 150_000);
    assert_eq!(UpdateSet::row_count(0.1), 150);
    // dbgen rounds the number of orders down to a multiple of 10,000 first
    assert_eq!(UpdateSet::row_count(0.05), 70);
    assert_eq!(UpdateSet::row_count(0.01), 10);
    assert_eq!(UpdateSet::row_count(0.001), 0);
    assert_eq!(OrderGenerator::update_set(0.001, 1, 1, 1).iter().count(), 0);
    assert_eq!(
        LineItemGenerator::update_set(0.001, 1, 1, 1).iter().count(),
        0
    );
    assert_eq!(DeleteGenerator::new(0.001, 1).iter().count(), 0);
}

/// Update sets continue the random streams of the base data
#[test]
fn test_update_set_continues_base_data() {
    let sf = 0.01;
    let base_rows = OrderGenerator::calculate_row_count(sf, 1, 1);
    let rows = UpdateSet::row_count(sf);
    let comments = |orders: Vec<_>| -> Vec<String> {
        orders
            .into_iter()
            .map(|o: tpchgen::generators::Order| o.o_comment.to_string())
            .collect()
    };
    // A base data generator for twice the scale factor generates the same
    // random values after the base data, but with different keys
    let continued = comments(
        OrderGenerator::new(2.0 * sf, 1, 1)
            .iter()
            .skip(base_rows as usize)
            .take(2 * rows as usize)
            .collect(),
    );
    let mut updates = comments(OrderGenerator::update_set(sf, 1, 1, 1).iter().collect());
    updates.extend(comments(
        OrderGenerator::update_set(sf, 2, 1, 1).iter().collect(),
    ));
    assert_eq!(continued, updates);
}

struct TestIntoIterator<G>
where
    G: IntoIterator,
    G::Item: std::fmt::Display,
{
    generator: Option<G>,
}

impl<G> TestIntoIterator<G>
where
    G: IntoIterator,
    G::Item: std::fmt::Display,
{
    fn new(generator: G) -> Self {
        Self {
            generator: Some(generator),
        }
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn to_string_vec(&mut self, take_num: i32) -> Vec<String> {
        if let Some(generator) = self.generator.take() {
            generator
                .into_iter()
                .take(take_num as usize)
                .map(|item| item.to_string())
                .collect()
        } else {
            vec![]
        }
    }
}

#[test]
fn test_nation_into_iter() {
    {
        assert_eq!(
            TestIntoIterator::new(NationGenerator::default())
                .to_string_vec(5)
                .len(),
            5
        );
    }
    {
        let nation = NationGenerator::default();
        assert_eq!(TestIntoIterator::new(nation).to_string_vec(5).len(), 5);
    }
}

#[test]
fn test_region_into_iter() {
    {
        assert_eq!(
            TestIntoIterator::new(RegionGenerator::default())
                .to_string_vec(5)
                .len(),
            5
        );
    }
    {
        let nation = RegionGenerator::default();
        assert_eq!(TestIntoIterator::new(nation).to_string_vec(5).len(), 5);
    }
}

#[test]
fn test_part_into_iter() {
    {
        assert_eq!(
            TestIntoIterator::new(PartGenerator::new(0.01, 1, 1))
                .to_string_vec(5)
                .len(),
            5
        );
    }
    {
        let nation = PartGenerator::new(0.01, 1, 1);
        assert_eq!(TestIntoIterator::new(nation).to_string_vec(5).len(), 5);
    }
}

#[test]
fn test_supplier_into_iter() {
    {
        assert_eq!(
            TestIntoIterator::new(SupplierGenerator::new(0.01, 1, 1))
                .to_string_vec(5)
                .len(),
            5
        );
    }
    {
        let nation = SupplierGenerator::new(0.01, 1, 1);
        assert_eq!(TestIntoIterator::new(nation).to_string_vec(5).len(), 5);
    }
}

#[test]
fn test_partsupp_into_iter() {
    {
        assert_eq!(
            TestIntoIterator::new(PartSuppGenerator::new(0.01, 1, 1))
                .to_string_vec(5)
                .len(),
            5
        );
    }
    {
        let nation = PartSuppGenerator::new(0.01, 1, 1);
        assert_eq!(TestIntoIterator::new(nation).to_string_vec(5).len(), 5);
    }
}

#[test]
fn test_customer_into_iter() {
    {
        assert_eq!(
            TestIntoIterator::new(CustomerGenerator::new(0.01, 1, 1))
                .to_string_vec(5)
                .len(),
            5
        );
    }
    {
        let nation = CustomerGenerator::new(0.01, 1, 1);
        assert_eq!(TestIntoIterator::new(nation).to_string_vec(5).len(), 5);
    }
}

#[test]
fn test_orders_into_iter() {
    {
        assert_eq!(
            TestIntoIterator::new(OrderGenerator::new(0.01, 1, 1))
                .to_string_vec(5)
                .len(),
            5
        );
    }
    {
        let nation = OrderGenerator::new(0.01, 1, 1);
        assert_eq!(TestIntoIterator::new(nation).to_string_vec(5).len(), 5);
    }
}

#[test]
fn test_lineitem_into_iter() {
    {
        assert_eq!(
            TestIntoIterator::new(LineItemGenerator::new(0.01, 1, 1))
                .to_string_vec(5)
                .len(),
            5
        );
    }
    {
        let nation = LineItemGenerator::new(0.01, 1, 1);
        assert_eq!(TestIntoIterator::new(nation).to_string_vec(5).len(), 5);
    }
}
