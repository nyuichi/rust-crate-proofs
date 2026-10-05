(* Solver-free replay of an exact transformation path in the original task.
   No printed Why3 text is parsed again; no successful sibling is transformed. *)
open Why3

let write_task driver path task =
  let channel = open_out_bin path in
  let formatter = Format.formatter_of_out_channel channel in
  Driver.print_task driver formatter task;
  Format.pp_print_flush formatter ();
  close_out channel

let () =
  try
    if Array.length Sys.argv <> 5 then
      failwith "usage: replay INPUT.coma GOAL CHILD_PATH_OR_DASH OUTPUT_DIR";
    let input = Sys.argv.(1) and goal = Sys.argv.(2) in
    let path = if Sys.argv.(3) = "-" then [] else
      List.map int_of_string (String.split_on_char ',' Sys.argv.(3)) in
    let output = Sys.argv.(4) in
    let config = Whyconf.read_config (Some "/workspace/proof-tools/config/creusot/why3.conf") in
    let main = Whyconf.get_main config in
    Whyconf.load_plugins main;
    let env = Env.create_env
      ("/workspace/proof-tools/creusot-data/share/why3find/packages/creusot" :: Whyconf.loadpath main) in
    let driver = Driver.load_driver_file_and_extras main env ~extra_dir:None "why3" [] in
    let theories, _ = Env.read_file Env.base_language env input in
    let theory = Wstdlib.Mstr.find "Coma" theories in
    let tasks = Task.split_theory theory None None in
    let matches = List.filter (fun task ->
      (Task.task_goal task).Decl.pr_name.Ident.id_string = goal) tasks in
    let root = match matches with [task] -> task | _ -> failwith "goal not unique" in
    write_task driver (Filename.concat output "root.why") root;
    let rec replay task depth indices =
      write_task driver (Filename.concat output (Printf.sprintf "parent-%d.why" depth)) task;
      let children = Trans.apply_transform "split_vc" env task in
      Printf.printf "depth=%d arity=%d\n%!" depth (List.length children);
      List.iteri (fun i child ->
        write_task driver
          (Filename.concat output (Printf.sprintf "depth-%d-child-%d.why" depth i)) child) children;
      match indices with
      | [] -> ()
      | index :: rest -> replay (List.nth children index) (depth + 1) rest
    in
    replay root 0 path
  with error ->
    Format.eprintf "%a@." Exn_printer.exn_printer error;
    exit 1
