use super::Fixture;
use gitio::RunOpts;

pub fn refs(fixture: &Fixture) -> String {
    let output = fixture.git.run(
        &["for-each-ref", "--sort=refname", "--format=%(refname) %(objectname)", "refs/mortar/"],
        &RunOpts { read_only: true, ..RunOpts::default() },
    ).unwrap();
    String::from_utf8(output.stdout).unwrap()
}
