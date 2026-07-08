//! Slint ↔ Calculator bridge.
//! Owns the UI handle, wires all callbacks, and pushes state on every change.

use std::{cell::RefCell, rc::Rc};
use slint::{ComponentHandle, SharedString, VecModel};

use crate::{calculator::Calculator, MainWindow, HistoryEntry};

pub fn run() -> Result<(), slint::PlatformError> {
    let ui   = MainWindow::new()?;
    let calc = Rc::new(RefCell::new(Calculator::new()));

    // Push initial state
    push_display(&ui, &calc.borrow());

    // button-pressed ----------------------------------------------------------
    {
        let ui_w = ui.as_weak();
        let c2   = calc.clone();
        ui.on_button_pressed(move |label| {
            c2.borrow_mut().handle_input(label.as_str());
            let ui = ui_w.upgrade().unwrap();
            push_display(&ui, &c2.borrow());
        });
    }

    // mode-changed ------------------------------------------------------------
    {
        let ui_w = ui.as_weak();
        let c2   = calc.clone();
        ui.on_mode_changed(move |mode| {
            c2.borrow_mut().set_mode(mode);
            let ui = ui_w.upgrade().unwrap();
            ui.set_current_mode(mode);
            push_display(&ui, &c2.borrow());
        });
    }

    // history-clicked ---------------------------------------------------------
    {
        let ui_w = ui.as_weak();
        let c2   = calc.clone();
        ui.on_history_clicked(move |idx| {
            let mut c = c2.borrow_mut();
            let (expr, result) = if let Some(entry) = c.history.get(idx as usize) {
                (entry.expr.clone(), entry.result.clone())
            } else {
                return;
            };

            c.handle_input("AC");
            for ch in result.chars() {
                c.handle_input(&ch.to_string());
            }
            c.display = result;
            c.expression = expr;

            let ui = ui_w.upgrade().unwrap();
            push_display(&ui, &c);
        });
    }

    ui.run()
}

fn push_display(ui: &MainWindow, c: &Calculator) {
    ui.set_display_text(ss(&c.display));
    ui.set_expr_text(ss(&c.expression));
    ui.set_deg_mode(c.degree_mode);
    ui.set_mem_indicator(if c.memory.has_value() { ss("M") } else { ss("") });

    // Programmer displays
    ui.set_active_base(ss(c.prog.base.name()));
    ui.set_hex_text(ss(&c.hex_text()));
    ui.set_dec_text(ss(&c.dec_text()));
    ui.set_oct_text(ss(&c.oct_text()));
    ui.set_bin_text(ss(&c.bin_text()));

    // Financial TVM
    ui.set_tvm_n(ss(&c.tvm.n_str()));
    ui.set_tvm_iy(ss(&c.tvm.iy_str()));
    ui.set_tvm_pv(ss(&c.tvm.pv_str()));
    ui.set_tvm_pmt(ss(&c.tvm.pmt_str()));
    ui.set_tvm_fv(ss(&c.tvm.fv_str()));
    ui.set_tvm_sel(ss(&c.tvm_sel));

    // History (rebuild model each time — bounded to 100 entries)
    let model = Rc::new(VecModel::<HistoryEntry>::default());
    for rec in c.history.all() {
        model.push(HistoryEntry {
            expr:   ss(&rec.expr),
            result: ss(&rec.result),
            ts:     ss(&rec.ts),
        });
    }
    ui.set_history(model.into());
}

#[inline]
fn ss(s: &str) -> SharedString { SharedString::from(s) }
