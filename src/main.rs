use anyhow::bail;
use structopt::StructOpt;
use crate::ids::{IDSTable, parse};

mod ids;

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
    }
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let opt = Opt::from_args();
    match opt.cmd {
        Command::Find { needles: needle_strs } => {
            let needles = {
                let mut needles = vec![];
                for needle_str in needle_strs {
                    let Ok(needle) = parse(&needle_str) else {
                        bail!("Cannot parse needle {}", needle_str);
                    };
                    needles.push(needle);
                }
                needles
            };
            let table = IDSTable::load_file("chai.txt")?;
            let result: Vec<_> = table.iter()
                .filter_map(|(k, tagged_ids)| {
                    if needles.iter().all(|needle| table.ids_has_subcomponent(&tagged_ids.ids, &needle.ids)) {
                        Some(k)
                    } else {
                        None
                    }
                })
                .collect();
            for k in result {
                println!("{}", k);
            }
        }
    }
    Ok(())
}
