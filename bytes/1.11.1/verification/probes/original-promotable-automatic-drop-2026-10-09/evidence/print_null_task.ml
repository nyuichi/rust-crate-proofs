open Why3
let () =
 let config = Whyconf.init_config None in
 let main = Whyconf.get_main config in
 Whyconf.load_plugins main;
 let env = Env.create_env ("/workspace/bytes-proof-tools/creusot-data/share/why3find/packages/creusot" :: Whyconf.loadpath main) in
 let theories,_ = Env.read_file ~format:"coma" Env.base_language env Sys.argv.(1) in
 let thy = Wstdlib.Mstr.find "Coma" theories in
 let tasks = Task.split_theory thy None None in
 let task = List.find (fun t -> (Task.task_goal t).Decl.pr_name.Ident.id_string = Sys.argv.(2)) tasks in
 let task = List.hd (Trans.apply_transform "compute_specified" env task) in
 let path = String.split_on_char ',' Sys.argv.(3) |> List.map int_of_string in
 let task = List.fold_left (fun task i -> List.nth (Trans.apply_transform "split_vc" env task) i) task path in
 Format.printf "%a@." Pretty.print_task task
