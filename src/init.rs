// Runtime bootstrap builder for bundled browser-side helper scripts.
use maud::{Markup, PreEscaped, Render, html};

/// Page-level runtime bootstrap for bundled browser-side helpers.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Init {
    surrealjs: bool,
    scoped_css: bool,
    signals: bool,
}

impl Init {
    /// Starts with an empty runtime bootstrap configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Includes the bundled Surreal runtime.
    pub fn surrealjs(mut self) -> Self {
        self.surrealjs = true;
        self
    }

    /// Includes the bundled css-scope-inline runtime.
    pub fn scoped_css(mut self) -> Self {
        self.scoped_css = true;
        self
    }

    /// Includes the bundled Signals runtime and adapter.
    pub fn signals(mut self) -> Self {
        self.signals = true;
        self
    }

    /// Builds the runtime bootstrap markup.
    pub fn build(self) -> Markup {
        let mut scripts = Vec::new();

        if self.surrealjs {
            scripts.push(sanitized_script(include_str!("../assets/surreal.js")));
        }
        if self.scoped_css {
            scripts.push(sanitized_script(include_str!("../assets/css-scope-inline.js")));
        }
        if self.signals {
            scripts.push(sanitized_script(include_str!("../assets/signals-core.min.js")));
            scripts.push(sanitized_script(include_str!("../assets/signals-adapter.js")));
        }

        html! {
            @for script_text in scripts {
                script { (PreEscaped(script_text)) }
            }
        }
    }

    /// Emits the full recommended runtime bundle.
    pub fn all() -> Markup {
        Self::new().surrealjs().scoped_css().signals().build()
    }
}

impl Render for Init {
    fn render(&self) -> Markup {
        (*self).build()
    }
}

fn sanitized_script(source: &str) -> String {
    source
        .replace("</script>", "<\\/script>")
        .replace("</SCRIPT>", "<\\/SCRIPT>")
        .replace("</Script>", "<\\/Script>")
}
