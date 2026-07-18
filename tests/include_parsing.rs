//! Tests for top-level `Include` (and `Match`) parsing.
//!
//! Regression coverage for the parser previously erroring on `Include`
//! directives that appear outside a `Host`/`Match` block
//! (azriel91/ssh_cfg#3).

use ssh_cfg::{SshConfigParser, SshOptionKey, SshSection};

#[test]
fn top_level_include_before_hosts_parses() {
    let contents = "\
Include config.d/foo

Host example
    HostName example.com
    User alice
";

    let ssh_config =
        SshConfigParser::parse_config_contents(contents).expect("expected config to parse");

    let sections = ssh_config.keys().cloned().collect::<Vec<_>>();
    assert_eq!(
        vec![
            SshSection::Include("config.d/foo".to_string()),
            SshSection::Host("example".to_string()),
        ],
        sections,
    );

    // The `Include` section carries no options of its own.
    let include_config = &ssh_config[&SshSection::Include("config.d/foo".to_string())];
    assert!(include_config.is_empty());

    // The following `Host` block is parsed normally.
    let host_config = &ssh_config[&SshSection::Host("example".to_string())];
    assert_eq!(
        Some(&"example.com".to_string()),
        host_config.get(&SshOptionKey::Hostname),
    );
    assert_eq!(
        Some(&"alice".to_string()),
        host_config.get(&SshOptionKey::User),
    );
}

#[test]
fn include_inside_host_block_is_an_option() {
    let contents = "\
Host example
    Include some/included/file
    User bob
";

    let ssh_config =
        SshConfigParser::parse_config_contents(contents).expect("expected config to parse");

    // No top-level `Include` section: the directive belongs to the host.
    let sections = ssh_config.keys().cloned().collect::<Vec<_>>();
    assert_eq!(vec![SshSection::Host("example".to_string())], sections);

    let host_config = &ssh_config[&SshSection::Host("example".to_string())];
    assert_eq!(
        Some(&"some/included/file".to_string()),
        host_config.get(&SshOptionKey::Include),
    );
    assert_eq!(
        Some(&"bob".to_string()),
        host_config.get(&SshOptionKey::User)
    );
}

#[test]
fn top_level_match_block_parses() {
    let contents = "\
Match host example
    User carol
";

    let ssh_config =
        SshConfigParser::parse_config_contents(contents).expect("expected config to parse");

    let match_config = &ssh_config[&SshSection::Match("host example".to_string())];
    assert_eq!(
        Some(&"carol".to_string()),
        match_config.get(&SshOptionKey::User),
    );
}

#[test]
fn option_before_host_or_match_is_an_error() {
    // A bare option with no preceding `Host`/`Match` is invalid.
    let contents = "User nobody\n";

    let error = SshConfigParser::parse_config_contents(contents)
        .expect_err("expected an option-before-host error");

    assert!(
        format!("{error:?}").contains("SshOptionBeforeHostOrMatch"),
        "unexpected error: {:?}",
        error,
    );
}

#[test]
fn ssh_section_display() {
    assert_eq!(
        "Include config.d/foo",
        SshSection::Include("config.d/foo".to_string()).to_string(),
    );
    assert_eq!(
        "Host example",
        SshSection::Host("example".to_string()).to_string(),
    );
    assert_eq!(
        "Match host example",
        SshSection::Match("host example".to_string()).to_string(),
    );
}
