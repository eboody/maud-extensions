use maud_extensions::js;

fn main() {
    js!(card_script, always, {
        me().class_add("ready");
    });
}
