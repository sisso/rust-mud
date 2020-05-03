use crate::game::container::Container;
use crate::game::system::{SystemCtx, Systems};
use crate::game::Outputs;
use crate::game::{system, GameCfg};
use commons::save::{Snapshot, SnapshotSupport};
use commons::DeltaTime;
use logs::*;

pub fn tick(
    cfg: &GameCfg,
    delta_time: DeltaTime,
    container: &mut Container,
    systems: &mut Systems,
    outputs: &mut dyn Outputs,
) {
    container.time.add(delta_time);

    if cfg.persistent && container.time.tick.as_u32() % 100 == 0 {
        debug!("tick {:?}", container.time);

        let mut snapshot = Snapshot::new();
        container.save_snapshot(&mut snapshot);
        snapshot.save_to_file(format!("/tmp/{}.save", cfg.profile).as_str());
        snapshot.save_to_file(
            format!(
                "/tmp/{}_{}.snapshot",
                cfg.profile,
                container.time.tick.as_u32()
            )
            .as_str(),
        );
    }

    let mut ctx = SystemCtx { container, outputs };

    // TODO: inputs
    systems.tick(&mut ctx);
    // TODO: after rum? trigger?
    // TODO: outputs
    container.triggers.clear();
}
