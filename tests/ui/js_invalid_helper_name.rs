use maud_extensions::js;

fn main() {
    js!("card-script", {
        me().class_add("ready");
    });
}
