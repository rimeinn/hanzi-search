use structopt::StructOpt;
use hanzi_search::{ids::IDSTable, search_find, search_match, search_pmatch};

// Embed the data file into the binary
const CHAI_DATA: &str = include_str!("../chai.txt");

#[derive(StructOpt, Debug)]
#[structopt(name = "hanzi-search")]
struct Opt {
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(StructOpt, Debug)]
enum Command {
    Find {
        needles: Vec<String>,
    },
    Match {
        pattern: String,
    },
    Pmatch {
        pattern: String,
    },
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let opt = Opt::from_args();
    let table = IDSTable::load_from_string(CHAI_DATA)?;

    match opt.cmd {
        Command::Find { needles } => {
            let result = search_find(&table, &needles)
                .map_err(|e| anyhow::anyhow!(e))?;
            for (k, t) in result {
                println!("{} {}", k, t);
            }
        }

        Command::Match { pattern } => {
            let result = search_match(&table, &pattern)
                .map_err(|e| anyhow::anyhow!(e))?;
            for (k, t) in result {
                println!("{} {}", k, t);
            }
        }

        Command::Pmatch { pattern } => {
            let result = search_pmatch(&table, &pattern)
                .map_err(|e| anyhow::anyhow!(e))?;
            for (k, t) in result {
                println!("{} {}", k, t);
            }
        }
    }
    Ok(())
}
