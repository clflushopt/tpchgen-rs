import sys
from tpchgen import rust

def main():
    rust.run_cli(sys.argv[1:])
