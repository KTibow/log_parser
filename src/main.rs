use regex::Regex;
use std::{env, fs};

fn main() {
    let log_to_read = env::args()
        .nth(1)
        .expect("Please provide a log file to read");
    let log_contents = fs::read_to_string(log_to_read).expect("Could not read log file");

    println!("**Info**");
    let forge_re =
        Regex::new(r"(?:Powered by Forge |MinecraftForge v|Forge Mod Loader version )([0-9\.]+)")
            .unwrap();
    if let Some(captures) = forge_re.captures(&log_contents) {
        let forge_version = captures.get(1).unwrap().as_str();
        print!("Forge {}. ", forge_version);
        if forge_version != "11.15.1.2318" {
            print!("Please [update](https://files.minecraftforge.net/net/minecraftforge/forge/index_1.8.9.html) to the latest version. ");
        }
    }
    let optifine_re = Regex::new(r"OptiFine_1\.8\.9_HD_U_([A-Z][0-9])").unwrap();
    if let Some(captures) = optifine_re.captures(&log_contents) {
        let optifine_version = captures.get(1).unwrap().as_str();
        print!("Optifine {}. ", optifine_version);
    }
    if log_contents.contains("FeatherOpt") {
        print!("Feather \"Client\" is not supported, ask discord.gg/feather. ");
    }
    let mod_list_marker = "States: 'U' = Unloaded 'L' = Loaded 'C' = Constructed 'H' = Pre-initialized 'I' = Initialized 'J' = Post-initialized 'A' = Available 'D' = Disabled 'E' = Errored";
    if log_contents.contains(mod_list_marker) {
        let mod_re = Regex::new(r"\| UCHIJA \| ([a-zA-Z ']+?)\s+\| .+?\s+\| (.+?)\s+\|").unwrap();
        let mods_used = log_contents
            .split(mod_list_marker)
            .nth(1)
            .unwrap()
            .split("\n\n")
            .nth(1)
            .unwrap()
            .split("\n")
            .skip(3)
            .filter_map(|line| {
                if let Some(captures) = mod_re.captures(line) {
                    let mod_id = captures.get(1).unwrap().as_str();
                    if mod_id == "mcp"
                        || mod_id == "Forge"
                        || mod_id == "FML"
                        || mod_id == "onecore"
                        || mod_id == "essential"
                    {
                        return None;
                    }
                    Some(mod_id)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        // Try to read from the file mods_in_skyclient.txt; if possible, make a list of each line.
        let mods_in_skyclient_read = fs::read_to_string("mods_in_skyclient.txt");
        if let Ok(mods_in_skyclient) = mods_in_skyclient_read {
            // Say {}/{} mods used are in Skyclient.
            let mods_in_skyclient = mods_in_skyclient.split("\n").collect::<Vec<_>>();
            let mut mods_used_that_are_in_skyclient = Vec::new();
            for mod_used in &mods_used {
                if mods_in_skyclient.contains(mod_used) {
                    mods_used_that_are_in_skyclient.push(mod_used);
                }
            }
            print!(
                "{}/{} mods used are in Skyclient. ",
                mods_used_that_are_in_skyclient.len(),
                mods_used.len()
            );
        } else {
            print!("{} mods used. ", mods_used.len());
        }
    }
    let time_re = Regex::new(r"(?m)^Time: (.+)").unwrap();
    if let Some(captures) = time_re.captures(&log_contents) {
        let time = captures.get(1).unwrap().as_str();
        print!("From {}. ", time);
    }
    println!("");

    let crash_data_contents =
        fs::read_to_string("crash_data.json").expect("Could not read crash data file");
    let crash_data: serde_json::Value =
        serde_json::from_str(&crash_data_contents).expect("Could not parse crash data");
    let mut recommendations_found = Vec::new();
    let mut solutions_found = Vec::new();
    for crash_info in crash_data.get("fixes").unwrap().as_array().unwrap() {
        let causes = crash_info.get("causes").unwrap().as_array().unwrap();
        let mut causes_match = true;
        for cause in causes {
            let method = cause.get("method").unwrap().as_str().unwrap();
            let value = cause.get("value").unwrap().as_str().unwrap();
            if method == "contains" {
                if !log_contents.contains(value) {
                    causes_match = false;
                }
            } else if method == "contains_not" {
                if log_contents.contains(value) {
                    causes_match = false;
                }
            } else if method == "regex" {
                let re = Regex::new(value).unwrap();
                if !re.is_match(&log_contents) {
                    causes_match = false;
                }
            } else {
                panic!("Unknown cause method: {}", method);
            }
        }
        if !causes_match {
            continue;
        }
        // Get fixtype as i64, and default to 1
        let info_type = crash_info
            .get("fixtype")
            .and_then(|fixtype| fixtype.as_i64())
            .unwrap_or(1);
        if info_type == 1 {
            solutions_found.push(crash_info.get("fix").unwrap().as_str().unwrap());
        } else if info_type == 2 {
            recommendations_found.push(crash_info.get("fix").unwrap().as_str().unwrap());
        }
    }
    if !recommendations_found.is_empty() {
        println!("**Recommendations**");
        for recommendation in &recommendations_found {
            println!("- {}", recommendation);
        }
    }
    if !solutions_found.is_empty() {
        println!("**Solutions**");
        for solution in &solutions_found {
            println!("- {}", solution);
        }
    }
}
