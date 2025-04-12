from .tpchgen import generate_tpch

def main():
    import argparse
    parser = argparse.ArgumentParser(description="TPC-H data generator")
    parser.add_argument("--scale", type=float, default=1.0, help="Scale factor")
    parser.add_argument("--output", type=str, default=".", help="Output directory")
    parser.add_argument("--format", type=str, default="csv", help="Output format (csv, parquet, etc.)")
    args = parser.parse_args()
    
    generate_tpch(args.scale, args.output, args.format)

if __name__ == "__main__":
    main()