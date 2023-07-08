mod utils;

use utils::*;

#[test]
fn init_quartz() {
    let quartz = Quartz::default();

    quartz.cmd(&["init"]).unwrap();

    println!("{:?}", quartz.tmpdir.join(".quartz"));

    assert!(quartz.dir().exists(), ".quartz was not created");
}
