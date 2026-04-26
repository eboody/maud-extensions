use maud_extensions::js;

fn main() {
    js!(card_script, {
        me().class_add("ready");
    }, trailing);
}
