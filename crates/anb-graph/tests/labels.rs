//! The two properties of the drawing a machine can settle: no two names the
//! map shows may share a patch of screen, and every tile it shows must be
//! big enough to point at.
//!
//! A name's place is the browser's answer — it depends on the width the font
//! actually gives an id and on the layout that width feeds — so the page is
//! opened in one, and the boxes it says it placed are read back out of the
//! document it hands over.

use anb_core::{Epic, Graph, GraphNode, GraphSlice};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// A hub every child waits on, with ids as long as a notebook grows them,
/// and beside it the tasks nothing waits on. One rank of long names is where
/// they overprint, so it is what the map has to hold apart; a task nothing
/// waits on is drawn at the smallest weight there is, so it is where a tile
/// stops being big enough to point at.
fn a_crowded_epic() -> Graph {
    let hub = "task.the-hub-every-piece-of-this-work-waits-on";
    let children: Vec<String> = (0..24)
        .map(|index| format!("task.a-long-name-of-the-kind-a-notebook-grows-{index:02}"))
        .collect();
    let mut nodes = vec![GraphNode {
        id: hub.to_owned(),
        state: "open".to_owned(),
        archived: false,
        title: Some("The hub".to_owned()),
        epic: Some(Epic {
            id: hub.to_owned(),
            closed: 3,
            total: children.len(),
            next: None,
        }),
        blocked_by: children.clone(),
        origin: None,
        fields: Vec::new(),
        body: String::new(),
        mentions: Vec::new(),
    }];
    nodes.extend(children.iter().map(|id| GraphNode {
        id: id.clone(),
        state: "open".to_owned(),
        archived: false,
        title: Some("A child".to_owned()),
        epic: None,
        blocked_by: Vec::new(),
        origin: Some(hub.to_owned()),
        fields: Vec::new(),
        body: String::new(),
        mentions: Vec::new(),
    }));
    nodes.extend((0..3).map(|index| GraphNode {
        id: format!("task.nothing-in-the-notebook-waits-on-this-{index}"),
        state: "open".to_owned(),
        archived: false,
        title: Some("A loner".to_owned()),
        epic: None,
        blocked_by: Vec::new(),
        origin: None,
        fields: Vec::new(),
        body: String::new(),
        mentions: Vec::new(),
    }));
    Graph {
        slice: GraphSlice::default(),
        nodes,
    }
}

#[test]
fn no_two_names_the_map_shows_share_a_patch_of_screen() {
    let Some(browser) = a_browser() else {
        eprintln!("skipped: no browser to open the map in — name one in ANB_BROWSER");
        return;
    };
    let home = tempfile::tempdir().expect("a directory to open the map from");
    let page = home.path().join("map.html");
    std::fs::write(&page, anb_graph::render(&a_crowded_epic())).unwrap();

    for arrangement in ["web", "layered"] {
        let placed = placed_names(&browser, &page, arrangement);
        assert!(
            placed.len() > 1,
            "{arrangement}: {} names placed — a map showing at most one proves nothing",
            placed.len()
        );
        for (index, one) in placed.iter().enumerate() {
            for other in &placed[index + 1..] {
                assert!(
                    !overlap(one, other),
                    "{arrangement}: two names share a patch of screen: {one:?} and {other:?}"
                );
            }
        }
    }
}

/// How much of the screen a tile has to cover before a hand can point at it
/// rather than aim for it.
const COMFORTABLE: f64 = 16.0;

/// A tile is drawn at the weight of its dependencies, so a leaf is a few
/// pixels across. What a pointer is tested against is a separate circle held
/// at a size a hand can hit, and a map whose tiles cannot be hit is a map
/// whose records cannot be opened.
#[test]
fn every_tile_the_map_shows_is_big_enough_to_point_at() {
    let Some(browser) = a_browser() else {
        eprintln!("skipped: no browser to open the map in — name one in ANB_BROWSER");
        return;
    };
    let home = tempfile::tempdir().expect("a directory to open the map from");
    let page = home.path().join("map.html");
    std::fs::write(&page, anb_graph::render(&a_crowded_epic())).unwrap();

    for arrangement in ["web", "layered"] {
        let document = dumped(&browser, &page, arrangement);
        let zoom = zoom_of(&document);
        let reaches = reaches(&document);
        assert!(
            reaches.len() > 1,
            "{arrangement}: {} tiles drawn — a map showing at most one proves nothing",
            reaches.len()
        );
        for reach in reaches {
            let across = reach * 2.0 * zoom;
            assert!(
                across >= COMFORTABLE,
                "{arrangement}: a tile is {across:.1}px across at a zoom of {zoom}, \
                 under the {COMFORTABLE}px a reader can point at"
            );
        }
    }
}

/// How far the map is zoomed out, read off the transform the page settled
/// on. A reach is written in the map's own units, and only the zoom turns
/// those into the pixels a hand has to hit.
fn zoom_of(document: &str) -> f64 {
    let (_, after) = document
        .split_once("id=\"viewport\"")
        .expect("the page has a viewport");
    let (_, scaled) = after[..after.find('>').expect("the element opens and closes")]
        .split_once("scale(")
        .expect("the viewport carries the zoom it settled on");
    scaled[..scaled.find(')').expect("the zoom closes")]
        .parse()
        .expect("the zoom is a number")
}

/// The radius of every circle a pointer is tested against, in the map's own
/// units, as the page left them.
fn reaches(document: &str) -> Vec<f64> {
    document
        .split("class=\"reach\" r=\"")
        .skip(1)
        .map(|rest| {
            rest[..rest.find('"').expect("the radius closes")]
                .parse()
                .expect("a radius is a number")
        })
        .collect()
}

/// One name's ground as the browser laid it out: left, top, width, height,
/// in the pixels a reader sees.
type Ground = [f64; 4];

fn overlap(one: &Ground, other: &Ground) -> bool {
    one[0] < other[0] + other[2]
        && other[0] < one[0] + one[2]
        && one[1] < other[1] + other[3]
        && other[1] < one[1] + one[3]
}

/// The ground each name took, read out of the document a browser returns
/// once the map has arranged itself.
fn placed_names(browser: &Path, page: &Path, arrangement: &str) -> Vec<Ground> {
    let document = dumped(browser, page, arrangement);
    document
        .split("data-box=\"")
        .skip(1)
        .map(|rest| {
            let numbers: Vec<f64> = rest[..rest.find('"').expect("the attribute closes")]
                .split(',')
                .map(|value| value.parse().expect("a ground is four numbers"))
                .collect();
            Ground::try_from(numbers).expect("a ground is four numbers")
        })
        .collect()
}

/// How long the browser has to open one page. A wedged one would otherwise
/// hold the whole suite open with nothing to show for it.
const PATIENCE: Duration = Duration::from_secs(60);

fn dumped(browser: &Path, page: &Path, arrangement: &str) -> String {
    let written = page
        .parent()
        .expect("the map sits in a directory")
        .join(format!("dump-{arrangement}.html"));
    let mut opening = Command::new(browser)
        .args([
            "--headless",
            "--disable-gpu",
            "--no-sandbox",
            // The layout is fitted to the viewport, so the viewport is
            // stated rather than left to whatever the host defaults to.
            "--window-size=1400,900",
            "--virtual-time-budget=10000",
            "--dump-dom",
        ])
        .arg(format!("file://{}#{arrangement}", page.display()))
        .stdout(std::fs::File::create(&written).expect("a file to dump into"))
        .stderr(Stdio::null())
        .spawn()
        .expect("the browser runs");

    let deadline = Instant::now() + PATIENCE;
    while opening
        .try_wait()
        .expect("the browser is waited on")
        .is_none()
    {
        assert!(
            Instant::now() < deadline,
            "the browser did not open the map within {PATIENCE:?}"
        );
        std::thread::sleep(Duration::from_millis(50));
    }
    std::fs::read_to_string(&written).expect("the browser wrote the document")
}

/// A browser to open the map in: the one named for this run, else the first
/// of the usual ones that answers.
fn a_browser() -> Option<PathBuf> {
    if let Some(named) = std::env::var_os("ANB_BROWSER") {
        return Some(PathBuf::from(named));
    }
    [
        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
        "/Applications/Chromium.app/Contents/MacOS/Chromium",
        "google-chrome",
        "chromium",
        "chromium-browser",
    ]
    .into_iter()
    .map(PathBuf::from)
    .find(|candidate| {
        Command::new(candidate)
            .arg("--version")
            .output()
            .is_ok_and(|answered| answered.status.success())
    })
}
