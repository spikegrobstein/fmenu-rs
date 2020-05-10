use std::env;
use std::fs;

use std::os::unix::fs::PermissionsExt;
use futures::future::join_all;

use anyhow::Result;

fn main() -> Result<()> {
    let path = env::var("PATH").expect("PATH not set");

    let mut elements: Vec<String> = path.split(":")
        .map(|s| s.to_string())
        .collect::<Vec<String>>();

    elements.sort();
    elements.dedup();

    // now let's get all of the executables
    
    smol::run(async {
        let mut futures = vec![];

        for e in elements {
            futures.push(executables_from_dir(e));
        }

        let results = join_all(futures).await;

        for r in results {
            for p in r? {
                println!("{}", p);
            }
        }


        Ok(())
    })
}

async fn executables_from_dir(dir: String) -> Result<Vec<String>> {
    let list = fs::read_dir(dir)?;

    let executables = list.filter(move |f| {
        if let Ok(f) = f {
            let metadata = f.metadata().unwrap();
            let perm = metadata.permissions();
            perm.mode() & 0o111 != 0
        } else {
            false
        }
    });

    Ok(executables.map(|e| e.unwrap().file_name().into_string().unwrap()).collect())
}
