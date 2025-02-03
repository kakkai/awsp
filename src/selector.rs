#[cfg(test)]
mod tests {

    use super::*;
    use std::env;

    #[test]
    fn select_profile_with_selection() {
        select_profile("ped");
        let result = env::var("AWS_PROFILE").unwrap();
        let expect = String::from("ped");
        assert_eq!(expect, result);
    }

    #[test]
    fn select_region_with_selection() {
        select_region("ped");
        let result = env::var("AWS_DEFAULT_REGION").unwrap();
        let expect = String::from("ped");
        assert_eq!(expect, result);
    }

    #[test]
    fn parse_convert_to_map_test() {
        let mut map = HashMap::new();
        map.insert(String::from("key_1"), "ABC");
        map.insert(String::from("key_2"), "50");
        map.insert(String::from("key_3"), "value");
        let result = to_key_list(&map);

        assert!(result.iter().any(|&key| key == "key_1"));
        assert!(result.iter().any(|&key| key == "key_2"));
        assert!(result.iter().any(|&key| key == "key_3"));
    }

    // Flaky test
    // #[test]
    // fn parse_default_env_no_value() {
    //     let result = default_env("CHECK");
    //     let expect = String::from("");
    //     assert_eq!(expect, result);
    // }

    #[test]
    fn parse_default_env_has_value() {
        env::set_var("CHECK", "value");
        let result = default_env("CHECK");
        let expect = String::from("value");
        assert_eq!(expect, result);
    }
}

use crate::cmdline::Opt;

use awsp::file::config::{create_profile_config_map_from, get_aws_config_file_path};

use dialoguer::{theme::ColorfulTheme, Select};
use std::env;
use std::path::PathBuf;
use std::process::Command;
use std::{collections::HashMap, process};
use sysinfo::{get_current_pid, Pid, System};

const REGIONS_DISPLAY: &[&str] = &[
    "af-south-1     | Cape Town",
    "ap-east-1      | Hong Kong",
    "ap-northeast-1 | Tokyo",
    "ap-northeast-2 | Seoul",
    "ap-northeast-3 | Osaka-Local",
    "ap-south-1     | Mumbai",
    "ap-south-2     | Hyderabad",
    "ap-southeast-1 | Singapore",
    "ap-southeast-2 | Sydney",
    "ap-southeast-3 | Jakarta",
    "ap-southeast-4 | Melbourne",
    "ap-southeast-5 | Malaysia",
    "ap-southeast-7 | Thailand",
    "ca-central-1   | Canada (Central)",
    "ca-west-1      | Calgary",
    "cn-north-1     | Beijing",
    "cn-nortwest-1  | Ningxia",
    "eu-central-1   | Frankfurt",
    "eu-central-2   | Zurich",
    "eu-north-1     | Stockholm",
    "eu-south-1     | Milan",
    "eu-south-2     | Spain",
    "eu-west-1      | Ireland",
    "eu-west-2      | London",
    "eu-west-3      | Paris",
    "il-central-1   | Tel Aviv",
    "me-central-1   | UAE",
    "me-south-1     | Bahrain",
    "mx-central-1   | Mexico (Central)",
    "sa-east-1      | São Paulo",
    "us-east-1      | N. Virginia",
    "us-east-2      | Ohio",
    "us-gov-east-1  | AWS GovCloud (US-East)",
    "us-gov-west-1  | AWS GovCloud (US-West)",
    "us-west-1      | N. California",
    "us-west-2      | Oregon"
];

const REGIONS: &[&str] = &[
    "af-south-1",
    "ap-east-1",
    "ap-northeast-1",
    "ap-northeast-2",
    "ap-northeast-3",
    "ap-south-1",
    "ap-south-2",
    "ap-southeast-1",
    "ap-southeast-2",
    "ap-southeast-3",
    "ap-southeast-4",
    "ap-southeast-5",
    "ap-southeast-7",
    "ca-central-1",
    "ca-west-1",
    "cn-north-1",
    "cn-nortwest-1",
    "eu-central-1",
    "eu-central-2",
    "eu-north-1",
    "eu-south-1",
    "eu-south-2",
    "eu-west-1",
    "eu-west-2",
    "eu-west-3",
    "il-central-1",
    "me-central-1",
    "me-south-1",
    "mx-central-1",
    "sa-east-1",
    "us-east-1",
    "us-east-2",
    "us-gov-east-1",
    "us-gov-west-1",
    "us-west-1",
    "us-west-2"
];

const AWS_DEFAULT_PROFILE: &str = "AWS_PROFILE";
const AWS_DEFAULT_REGION: &str = "AWS_DEFAULT_REGION";
const VERSION: &str = env!("CARGO_PKG_VERSION");

// TODO Error Handler
// pub fn run(opt: &Opt) -> Result<(), Box<dyn Error>> {
pub fn run(opt: &Opt) {
    if opt.version {
        print!("\nawsp: ");
        green_ln!("{}\n", VERSION);
        process::exit(1);
    } else if opt.region {
        region_menu();
    } else {
        profile_menu();
        region_menu();
    }

    display_selected();

    exec_process();

    // TODO Error Handler
    // Ok(())
}

fn display_selected() {
    // clear screen charactor
    print!("{esc}c", esc = 27 as char);
    green!("\n ->");
    print!("  Profile: ");
    green!("{}", default_env("AWS_PROFILE"));
    print!(" | Region: ");
    green_ln!("{} \n", default_env("AWS_DEFAULT_REGION"));
}

fn profile_menu() {
    let location = get_aws_config_file_path().unwrap();
    let config_file = create_profile_config_map_from(location.as_path()).unwrap();
    let profile_list = to_key_list(&config_file);
    let profile_list = profile_list.as_slice();
    let default_profile = default_env("AWS_PROFILE");
    let display_prompt = format!("profile (current: {} )", default_profile);
    let selection = display(display_prompt, profile_list, 0);
    select_profile(profile_list[selection]);
}

fn region_menu() {
    let default_region = default_env("AWS_DEFAULT_REGION");
    let display_prompt = format!("region (current: {} )", default_region);
    let selection = display(display_prompt, REGIONS_DISPLAY, 0);
    select_region(REGIONS[selection]);
}

fn exec_process() {
    let current_pid = get_current_pid().ok().unwrap();
    Command::new(find_shell(current_pid).unwrap())
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
    terminate_parent_process(current_pid);
}

fn default_env(env: &str) -> String {
    match env::var(env) {
        Ok(env) => env,
        Err(_) => String::from(""),
    }
}

fn to_key_list<K, V>(map: &HashMap<K, V>) -> Vec<&K> {
    let mut key_list = Vec::new();

    for key in map.keys() {
        key_list.push(key);
    }

    key_list
}

fn find_shell(current_pid: Pid) -> Option<PathBuf> {
    let s = System::new_all();
    let current_process = s.process(current_pid)?;
    let parent_pid = current_process.parent()?;
    let parent_process = s.process(parent_pid)?;
    let shell_path = parent_process.exe().iter().collect();
    Some(shell_path)
}

fn terminate_parent_process(pid: Pid) {
    let s = System::new_all();
    let current_process = s.process(pid).unwrap();
    let parent_pid = current_process.parent().unwrap();
    let parent_process = s.process(parent_pid).unwrap();
    parent_process.kill();
}

fn display<T: ToString>(display_prompt: String, list: &[T], default: usize) -> usize {
    Select::with_theme(&ColorfulTheme::default())
        .with_prompt(display_prompt)
        .default(default)
        .items(list)
        .interact()
        .unwrap()
}

fn select_profile(profile: &str) {
    env::set_var(AWS_DEFAULT_PROFILE, profile);
}

fn select_region(region: &str) {
    env::set_var(AWS_DEFAULT_REGION, region);
}

// TODO manage stack process when run awsp multiple
// fn is_terminate_previous_process() -> Option<bool> {
//     let s = System::new_all();
//     let current_pid = get_current_pid().ok()?;
//     let current_process = s.process(current_pid)?;
//     let current_path = current_process.exe();
//     let parent_pid = current_process.parent()?;
//     let parent_process = s.process(parent_pid)?;
//     let parent_of_parent_pid = parent_process.parent()?;
//     let parent_of_parent_process = s.process(parent_of_parent_pid)?;
//     let parent_of_parent_path = parent_of_parent_process.exe();
//     let current_path = current_path.to_str().unwrap();
//     let parent_of_parent_path = parent_of_parent_path.to_str().unwrap();
//     dbg!(current_path);
//     dbg!(parent_of_parent_path);
//     if current_path.eq(parent_of_parent_path) {
//         parent_of_parent_process.kill(Signal::Kill);
//         return Some(true);
//     }
//     Some(false)
// }
