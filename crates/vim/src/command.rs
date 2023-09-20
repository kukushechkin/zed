use command_palette::CommandInterceptResult;
use editor::{SortLinesCaseInsensitive, SortLinesCaseSensitive};
use gpui::{impl_actions, Action, AppContext};
use serde_derive::Deserialize;
use workspace::{SaveBehavior, Workspace};

use crate::{
    motion::{EndOfDocument, Motion},
    normal::{
        move_cursor,
        search::{FindCommand, ReplaceCommand},
        JoinLines,
    },
    state::Mode,
    Vim,
};

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct GoToLine {
    pub line: u32,
}

impl_actions!(vim, [GoToLine]);

pub fn init(cx: &mut AppContext) {
    cx.add_action(|_: &mut Workspace, action: &GoToLine, cx| {
        Vim::update(cx, |vim, cx| {
            vim.switch_mode(Mode::Normal, false, cx);
            move_cursor(vim, Motion::StartOfDocument, Some(action.line as usize), cx);
        });
    });
}

pub fn command_interceptor(mut query: &str, _: &AppContext) -> Option<CommandInterceptResult> {
    // Note: this is a very poor simulation of vim's command palette.
    // In the future we should adjust it to handle parsing range syntax,
    // and then calling the appropriate commands with/without ranges.
    //
    // We also need to support passing arguments to commands like :w
    // (ideally with filename autocompletion).
    //
    // For now, you can only do a replace on the % range, and you can
    // only use a specific line number range to "go to line"
    while query.starts_with(":") {
        query = &query[1..];
    }

    let (name, action) = match query {
        // save and quit
        "w" | "wr" | "wri" | "writ" | "write" => (
            "write",
            workspace::Save {
                save_behavior: Some(SaveBehavior::PromptOnConflict),
            }
            .boxed_clone(),
        ),
        "w!" | "wr!" | "wri!" | "writ!" | "write!" => (
            "write!",
            workspace::Save {
                save_behavior: Some(SaveBehavior::SilentlyOverwrite),
            }
            .boxed_clone(),
        ),
        "q" | "qu" | "qui" | "quit" => (
            "quit",
            workspace::CloseActiveItem {
                save_behavior: Some(SaveBehavior::PromptOnWrite),
            }
            .boxed_clone(),
        ),
        "q!" | "qu!" | "qui!" | "quit!" => (
            "quit!",
            workspace::CloseActiveItem {
                save_behavior: Some(SaveBehavior::DontSave),
            }
            .boxed_clone(),
        ),
        "wq" => (
            "wq",
            workspace::CloseActiveItem {
                save_behavior: Some(SaveBehavior::PromptOnConflict),
            }
            .boxed_clone(),
        ),
        "wq!" => (
            "wq!",
            workspace::CloseActiveItem {
                save_behavior: Some(SaveBehavior::SilentlyOverwrite),
            }
            .boxed_clone(),
        ),
        "x" | "xi" | "xit" | "exi" | "exit" => (
            "exit",
            workspace::CloseActiveItem {
                save_behavior: Some(SaveBehavior::PromptOnConflict),
            }
            .boxed_clone(),
        ),
        "x!" | "xi!" | "xit!" | "exi!" | "exit!" => (
            "exit!",
            workspace::CloseActiveItem {
                save_behavior: Some(SaveBehavior::SilentlyOverwrite),
            }
            .boxed_clone(),
        ),
        "wa" | "wal" | "wall" => (
            "wall",
            workspace::SaveAll {
                save_behavior: Some(SaveBehavior::PromptOnConflict),
            }
            .boxed_clone(),
        ),
        "wa!" | "wal!" | "wall!" => (
            "wall!",
            workspace::SaveAll {
                save_behavior: Some(SaveBehavior::SilentlyOverwrite),
            }
            .boxed_clone(),
        ),
        "qa" | "qal" | "qall" | "quita" | "quital" | "quitall" => (
            "quitall",
            workspace::CloseAllItemsAndPanes {
                save_behavior: Some(SaveBehavior::PromptOnWrite),
            }
            .boxed_clone(),
        ),
        "qa!" | "qal!" | "qall!" | "quita!" | "quital!" | "quitall!" => (
            "quitall!",
            workspace::CloseAllItemsAndPanes {
                save_behavior: Some(SaveBehavior::DontSave),
            }
            .boxed_clone(),
        ),
        "xa" | "xal" | "xall" => (
            "xall",
            workspace::CloseAllItemsAndPanes {
                save_behavior: Some(SaveBehavior::PromptOnConflict),
            }
            .boxed_clone(),
        ),
        "xa!" | "xal!" | "xall!" => (
            "xall!",
            workspace::CloseAllItemsAndPanes {
                save_behavior: Some(SaveBehavior::SilentlyOverwrite),
            }
            .boxed_clone(),
        ),
        "wqa" | "wqal" | "wqall" => (
            "wqall",
            workspace::CloseAllItemsAndPanes {
                save_behavior: Some(SaveBehavior::PromptOnConflict),
            }
            .boxed_clone(),
        ),
        "wqa!" | "wqal!" | "wqall!" => (
            "wqall!",
            workspace::CloseAllItemsAndPanes {
                save_behavior: Some(SaveBehavior::SilentlyOverwrite),
            }
            .boxed_clone(),
        ),
        "cq" | "cqu" | "cqui" | "cquit" | "cq!" | "cqu!" | "cqui!" | "cquit!" => {
            ("cquit!", zed_actions::Quit.boxed_clone())
        }

        // pane management
        "sp" | "spl" | "spli" | "split" => ("split", workspace::SplitUp.boxed_clone()),
        "vs" | "vsp" | "vspl" | "vspli" | "vsplit" => {
            ("vsplit", workspace::SplitLeft.boxed_clone())
        }
        "new" => (
            "new",
            workspace::NewFileInDirection(workspace::SplitDirection::Up).boxed_clone(),
        ),
        "vne" | "vnew" => (
            "vnew",
            workspace::NewFileInDirection(workspace::SplitDirection::Left).boxed_clone(),
        ),
        "tabe" | "tabed" | "tabedi" | "tabedit" => ("tabedit", workspace::NewFile.boxed_clone()),
        "tabnew" => ("tabnew", workspace::NewFile.boxed_clone()),

        "tabn" | "tabne" | "tabnex" | "tabnext" => {
            ("tabnext", workspace::ActivateNextItem.boxed_clone())
        }
        "tabp" | "tabpr" | "tabpre" | "tabprev" | "tabprevi" | "tabprevio" | "tabpreviou"
        | "tabprevious" => ("tabprevious", workspace::ActivatePrevItem.boxed_clone()),
        "tabN" | "tabNe" | "tabNex" | "tabNext" => {
            ("tabNext", workspace::ActivatePrevItem.boxed_clone())
        }
        "tabc" | "tabcl" | "tabclo" | "tabclos" | "tabclose" => (
            "tabclose",
            workspace::CloseActiveItem {
                save_behavior: Some(SaveBehavior::PromptOnWrite),
            }
            .boxed_clone(),
        ),

        // quickfix / loclist (merged together for now)
        "cl" | "cli" | "clis" | "clist" => ("clist", diagnostics::Deploy.boxed_clone()),
        "cc" => ("cc", editor::Hover.boxed_clone()),
        "ll" => ("ll", editor::Hover.boxed_clone()),
        "cn" | "cne" | "cnex" | "cnext" => ("cnext", editor::GoToDiagnostic.boxed_clone()),
        "lne" | "lnex" | "lnext" => ("cnext", editor::GoToDiagnostic.boxed_clone()),

        "cpr" | "cpre" | "cprev" | "cprevi" | "cprevio" | "cpreviou" | "cprevious" => {
            ("cprevious", editor::GoToPrevDiagnostic.boxed_clone())
        }
        "cN" | "cNe" | "cNex" | "cNext" => ("cNext", editor::GoToPrevDiagnostic.boxed_clone()),
        "lp" | "lpr" | "lpre" | "lprev" | "lprevi" | "lprevio" | "lpreviou" | "lprevious" => {
            ("lprevious", editor::GoToPrevDiagnostic.boxed_clone())
        }
        "lN" | "lNe" | "lNex" | "lNext" => ("lNext", editor::GoToPrevDiagnostic.boxed_clone()),

        // modify the buffer (should accept [range])
        "j" | "jo" | "joi" | "join" => ("join", JoinLines.boxed_clone()),
        "d" | "de" | "del" | "dele" | "delet" | "delete" | "dl" | "dell" | "delel" | "deletl"
        | "deletel" | "dp" | "dep" | "delp" | "delep" | "deletp" | "deletep" => {
            ("delete", editor::DeleteLine.boxed_clone())
        }
        "sor" | "sor " | "sort" | "sort " => ("sort", SortLinesCaseSensitive.boxed_clone()),
        "sor i" | "sort i" => ("sort i", SortLinesCaseInsensitive.boxed_clone()),

        // goto (other ranges handled under _ => )
        "$" => ("$", EndOfDocument.boxed_clone()),

        _ => {
            if query.starts_with("/") || query.starts_with("?") {
                (
                    query,
                    FindCommand {
                        query: query[1..].to_string(),
                        backwards: query.starts_with("?"),
                    }
                    .boxed_clone(),
                )
            } else if query.starts_with("%") {
                (
                    query,
                    ReplaceCommand {
                        query: query.to_string(),
                    }
                    .boxed_clone(),
                )
            } else if let Ok(line) = query.parse::<u32>() {
                (query, GoToLine { line }.boxed_clone())
            } else {
                return None;
            }
        }
    };

    let string = ":".to_owned() + name;
    let positions = generate_positions(&string, query);

    Some(CommandInterceptResult {
        action,
        string,
        positions,
    })
}

fn generate_positions(string: &str, query: &str) -> Vec<usize> {
    let mut positions = Vec::new();
    let mut chars = query.chars().into_iter();

    let Some(mut current) = chars.next() else {
        return positions;
    };

    for (i, c) in string.chars().enumerate() {
        if c == current {
            positions.push(i);
            if let Some(c) = chars.next() {
                current = c;
            } else {
                break;
            }
        }
    }

    positions
}

#[cfg(test)]
mod test {
    use std::path::Path;

    use crate::test::{NeovimBackedTestContext, VimTestContext};
    use gpui::{executor::Foreground, TestAppContext};
    use indoc::indoc;

    #[gpui::test]
    async fn test_command_basics(cx: &mut TestAppContext) {
        if let Foreground::Deterministic { cx_id: _, executor } = cx.foreground().as_ref() {
            executor.run_until_parked();
        }
        let mut cx = NeovimBackedTestContext::new(cx).await;

        cx.set_shared_state(indoc! {"
            ˇa
            b
            c"})
            .await;

        cx.simulate_shared_keystrokes([":", "j", "enter"]).await;

        // hack: our cursor positionining after a join command is wrong
        cx.simulate_shared_keystrokes(["^"]).await;
        cx.assert_shared_state(indoc! {
            "ˇa b
            c"
        })
        .await;
    }

    #[gpui::test]
    async fn test_command_goto(cx: &mut TestAppContext) {
        let mut cx = NeovimBackedTestContext::new(cx).await;

        cx.set_shared_state(indoc! {"
            ˇa
            b
            c"})
            .await;
        cx.simulate_shared_keystrokes([":", "3", "enter"]).await;
        cx.assert_shared_state(indoc! {"
            a
            b
            ˇc"})
            .await;
    }

    #[gpui::test]
    async fn test_command_replace(cx: &mut TestAppContext) {
        let mut cx = NeovimBackedTestContext::new(cx).await;

        cx.set_shared_state(indoc! {"
            ˇa
            b
            c"})
            .await;
        cx.simulate_shared_keystrokes([":", "%", "s", "/", "b", "/", "d", "enter"])
            .await;
        cx.assert_shared_state(indoc! {"
            a
            ˇd
            c"})
            .await;
        cx.simulate_shared_keystrokes([
            ":", "%", "s", ":", ".", ":", "\\", "0", "\\", "0", "enter",
        ])
        .await;
        cx.assert_shared_state(indoc! {"
            aa
            dd
            ˇcc"})
            .await;
    }

    #[gpui::test]
    async fn test_command_search(cx: &mut TestAppContext) {
        let mut cx = NeovimBackedTestContext::new(cx).await;

        cx.set_shared_state(indoc! {"
                ˇa
                b
                a
                c"})
            .await;
        cx.simulate_shared_keystrokes([":", "/", "b", "enter"])
            .await;
        cx.assert_shared_state(indoc! {"
                a
                ˇb
                a
                c"})
            .await;
        cx.simulate_shared_keystrokes([":", "?", "a", "enter"])
            .await;
        cx.assert_shared_state(indoc! {"
                ˇa
                b
                a
                c"})
            .await;
    }

    #[gpui::test]
    async fn test_command_write(cx: &mut TestAppContext) {
        let mut cx = VimTestContext::new(cx, true).await;
        let path = Path::new("/root/dir/file.rs");
        let fs = cx.workspace(|workspace, cx| workspace.project().read(cx).fs().clone());

        cx.simulate_keystrokes(["i", "@", "escape"]);
        cx.simulate_keystrokes([":", "w", "enter"]);

        assert_eq!(fs.load(&path).await.unwrap(), "@\n");

        fs.as_fake()
            .write_file_internal(path, "oops\n".to_string())
            .unwrap();

        // conflict!
        cx.simulate_keystrokes(["i", "@", "escape"]);
        cx.simulate_keystrokes([":", "w", "enter"]);
        let window = cx.window;
        assert!(window.has_pending_prompt(cx.cx));
        // "Cancel"
        window.simulate_prompt_answer(0, cx.cx);
        assert_eq!(fs.load(&path).await.unwrap(), "oops\n");
        assert!(!window.has_pending_prompt(cx.cx));
        // force overwrite
        cx.simulate_keystrokes([":", "w", "!", "enter"]);
        assert!(!window.has_pending_prompt(cx.cx));
        assert_eq!(fs.load(&path).await.unwrap(), "@@\n");
    }

    #[gpui::test]
    async fn test_command_quit(cx: &mut TestAppContext) {
        let mut cx = VimTestContext::new(cx, true).await;

        cx.simulate_keystrokes([":", "n", "e", "w", "enter"]);
        cx.workspace(|workspace, cx| assert_eq!(workspace.items(cx).count(), 2));
        cx.simulate_keystrokes([":", "q", "enter"]);
        cx.workspace(|workspace, cx| assert_eq!(workspace.items(cx).count(), 1));
    }
}
