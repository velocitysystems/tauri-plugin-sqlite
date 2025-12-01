fn main() {
   tauri_plugin::Builder::new(&[
      "load",
      "execute",
      "execute_transaction",
      "fetch_all",
      "fetch_one",
      "close",
      "close_all",
      "remove",
   ])
   .build();
}
