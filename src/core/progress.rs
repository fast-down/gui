use crate::ui::Progress;
use fast_down_ffi::fast_down::ProgressEntry;
use slint::{Model, ModelRc, VecModel};
use std::rc::Rc;

pub fn apply_progress_diff(
    model_handle: &ModelRc<Progress>,
    new_ranges: &[ProgressEntry],
    total_size: u64,
) -> ModelRc<Progress> {
    #![allow(clippy::needless_range_loop)]

    if total_size == 0 {
        return model_handle.clone();
    }
    let total_f = total_size as f32;
    let model = model_handle
        .as_any()
        .downcast_ref::<Rc<VecModel<Progress>>>()
        .cloned()
        .unwrap_or_default();

    let old_len = model.row_count();
    let new_len = new_ranges.len();

    for i in 0..old_len.min(new_len) {
        let r = &new_ranges[i];
        let new_val = Progress {
            start: r.start as f32 / total_f,
            width: (r.end - r.start) as f32 / total_f,
        };
        if model.row_data(i).as_ref() != Some(&new_val) {
            model.set_row_data(i, new_val);
        }
    }

    if new_len > old_len {
        for i in old_len..new_len {
            let r = &new_ranges[i];
            model.push(Progress {
                start: r.start as f32 / total_f,
                width: (r.end - r.start) as f32 / total_f,
            });
        }
    } else if old_len > new_len {
        for i in (new_len..old_len).rev() {
            model.remove(i);
        }
    }
    model.into()
}
