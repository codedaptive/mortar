use crate::{json, support::{fixture_partners::{partners, text, Partners}, raw, REQUEST, ZONE}};

#[test]
fn partner_cut_and_repeat() {
    let f = Partners::new();
    let out = f.cut(&["--with", "web"]);
    let body = text(&out);
    let report = json::check(out, "folder", 0, "made");
    let base = f.tip(REQUEST);
    assert_eq!(report.text("base"), base.as_str());
    assert_eq!(f.git.commit_info(&base).unwrap().parents, vec![f.base.clone(), f.web.clone()]);
    assert_eq!(f.local().rev_parse("HEAD").unwrap(), Some(base.clone()));
    assert!(body.contains(&partners(&f.web)), "{body}");
    assert!(body.find("\"paths\":").unwrap() < body.find("\"partners\":").unwrap());
    assert!(body.find("\"partners\":").unwrap() < body.find("\"build_output\":").unwrap());
    let record = folders::make::read_recorded_base(&f.git.git_dir(&f.request).unwrap()).unwrap().unwrap();
    assert_eq!(record.base, base);
    assert_eq!(record.partners.len(), 1);
    assert_eq!(record.partners[0].zone, "web");
    assert_eq!(record.partners[0].tip, f.web);
    let again = json::check(f.cut(&["--with", "web"]), "folder", 0, "made");
    assert_eq!(again.text("base"), base.as_str());
    assert_eq!(f.tip(REQUEST), base);
}

#[test]
fn partner_cut_reports_partner_already_in_zone() {
    let f = Partners::new();
    raw(&f.git, &["update-ref", ZONE, f.web.as_str()]);
    let zone = f.advance(ZONE, &[]);
    let out = f.cut(&["--with", "web"]); let body = text(&out);
    let report = json::check(out, "folder", 0, "made");
    assert_eq!(report.text("base"), zone.as_str());
    assert!(body.contains("\"partners\":[]"), "{body}");
    assert!(body.contains("\"partners_in_zone\":[\"web\"]"), "{body}");
    assert_eq!(report.strings("partners_in_zone"), &["web"]);
    assert!(body.find("\"partners\":").unwrap() < body.find("\"partners_in_zone\":").unwrap());
}

#[test]
fn partner_cut_conflict_makes_nothing() {
    let f = Partners::new();
    f.advance("refs/heads/zone/core", &[("Cargo.toml", b"core\n")]);
    f.advance("refs/heads/zone/web", &[("Cargo.toml", b"web\n")]);
    let report = json::check(f.cut(&["--with", "web"]), "folder", 3, "refused");
    assert_eq!(report.text("partner"), "web");
    assert!(report.strings("files").contains(&"Cargo.toml".to_string()));
    assert!(!f.request.exists());
    assert_eq!(f.git.read_ref(REQUEST).unwrap(), None);
}

macro_rules! refusal {
    ($name:ident, $words:expr, $exit:expr, $outcome:expr) => {
        #[test]
        fn $name() {
            let f = Partners::new();
            json::check(f.cut($words), "folder", $exit, $outcome);
            assert!(!f.request.exists());
            assert_eq!(f.git.read_ref(REQUEST).unwrap(), None);
        }
    };
}
refusal!(partner_cut_self_refused, &["--with", "core"], 2, "usage");
refusal!(partner_cut_duplicate_refused, &["--with", "web", "--with", "web"], 2, "usage");
refusal!(partner_cut_unknown_refused, &["--with", "nosuch"], 3, "refused");
