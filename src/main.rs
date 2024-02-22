fn main() -> anyhow::Result<()> {
    brash::bash_it(std::env::args().skip(1))
}
