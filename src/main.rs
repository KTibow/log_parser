use regex::Regex;
use std::{collections, env, fs};

fn main() {
    let log_to_read = env::args()
        .nth(1)
        .expect("Please provide a log file to read");
    let log_contents = fs::read_to_string(log_to_read).expect("Could not read log file");
    let mut recommendations_found: Vec<String> = Vec::new();
    let mut solutions_found: Vec<String> = Vec::new();

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
    let mut corrupt_mods_detected = Vec::new();
    if log_contents.contains(mod_list_marker) {
        let mod_re =
            Regex::new(r"(?:\| UC?H?I?[JE]?A?\s+\| ([a-zA-Z '_\-0-9]+?)\s+\| .+?\s+\| (.+?)\s+\||(?:UC?H?I?[JE]?A?\t)?([a-zA-Z '_\-0-9]+?)\{.+?\} \[.+?\] \((.+?)\) )").unwrap();
        let dash_re = Regex::new(r"^-+$").unwrap();
        let mods_used = log_contents
            .split(mod_list_marker)
            .nth(1)
            .unwrap()
            .split("Loaded coremods (and transformers):")
            .next()
            .unwrap()
            .split('\n')
            .filter_map(|line| {
                if let Some(captures) = mod_re.captures(line) {
                    let mod_id = captures
                        .get(1)
                        .or_else(|| captures.get(3))
                        .unwrap()
                        .as_str();
                    if mod_id == "mcp"
                        || mod_id == "Forge"
                        || mod_id == "FML"
                        || mod_id == "onecore"
                        || mod_id == "essential"
                        || mod_id == "ID"
                        || dash_re.is_match(mod_id)
                    {
                        return None;
                    }
                    let mod_file = captures
                        .get(2)
                        .or_else(|| captures.get(4))
                        .unwrap()
                        .as_str();
                    if mod_id == "null" {
                        corrupt_mods_detected.push(mod_file);
                    }
                    Some(mod_id)
                } else {
                    None
                }
            })
            .collect::<collections::HashSet<_>>();
        if mods_used.is_empty() {
            print!("Couldn't detect mods used. ");
        } else {
            // Try to read from the file mods_in_skyclient.txt; if possible, make a list of each line.
            let mods_in_skyclient_read = fs::read_to_string("mods_in_skyclient.txt");
            if let Ok(mods_in_skyclient) = mods_in_skyclient_read {
                // Say {}/{} mods used are in Skyclient.
                let mods_in_skyclient = mods_in_skyclient
                    .split('\n')
                    .collect::<collections::HashSet<_>>();
                let mods_used_that_are_in_skyclient = mods_in_skyclient.intersection(&mods_used);
                print!(
                    "{}/{} mods used are in Skyclient. ",
                    mods_used_that_are_in_skyclient.count(),
                    mods_used.len()
                );
            } else {
                print!("{} mods used. ", mods_used.len());
            }
        }
        if mods_used.contains(&"skyblockhud") && mods_used.contains(&"apec") {
            recommendations_found.push(
                "In general, selecting all the mods is a bad practice. \
                You selected both SkyblockHUD and Apec, which do the same thing."
                    .into(),
            );
        }
        if mods_used.contains(&"musicplayer") && mods_used.contains(&"craftify") {
            recommendations_found.push(
                "In general, selecting all the mods is a bad practice. \
                You selected both Music Player and Craftify, which do the same thing."
                    .into(),
            );
        }
        if !mods_used.contains(&"patcher") {
            recommendations_found.push(
                "You didn't select the Patcher mod. \
                Patcher fixes some bugs, and improves FPS."
                    .into(),
            );
        }
    }
    let time_re = Regex::new(r"(?m)^Time: (.+)").unwrap();
    if let Some(captures) = time_re.captures(&log_contents) {
        let time = captures.get(1).unwrap().as_str();
        print!("From {}. ", time);
    }
    println!();

    let crash_data_contents =
        fs::read_to_string("crash_data.json").expect("Could not read crash data file");
    let crash_data: serde_json::Value =
        serde_json::from_str(&crash_data_contents).expect("Could not parse crash data");
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
        let fix = crash_info
            .get("fix")
            .unwrap()
            .as_str()
            .unwrap()
            .replace("%pathindicator%", "`")
            .replace("%profileroot%", ".minecraft/skyclient")
            .replace("%gameroot%", ".minecraft");
        if info_type == 1 {
            solutions_found.push(fix);
        } else if info_type == 2 {
            recommendations_found.push(fix);
        }
    }
    for corrupt_mod in &corrupt_mods_detected {
        recommendations_found.push(format!("{} might be corrupt, try removing it", corrupt_mod));
    }
    if log_contents.contains("JVM Flags:") {
        let max_memory_re = Regex::new(r"JVM Flags: .*-Xmx([0-9]+)([GgMm]).*").unwrap();
        if let Some(captures) = max_memory_re.captures(&log_contents) {
            let max_memory_mb = captures.get(1).unwrap().as_str().parse::<u64>().unwrap()
                * match captures.get(2).unwrap().as_str() {
                    "G" => 1024,
                    "g" => 1024,
                    "M" => 1,
                    "m" => 1,
                    _ => panic!("Unknown memory unit"),
                };
            if max_memory_mb < 2048 {
                recommendations_found.push(format!(
                    "Try increasing the max memory to at least 2GB, currently {:.2}GB",
                    max_memory_mb / 1024
                ));
            } else if max_memory_mb > 4096 {
                recommendations_found.push(format!(
                    "Try decreasing the max memory to at most 4GB, currently {:.2}GB",
                    max_memory_mb / 1024
                ));
            }
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
