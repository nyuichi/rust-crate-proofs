open Why3

(* This plugin is intentionally bound to one generated VC shape.  The hash is
   Term.t_hash_strict of the complete post-WP formula in task-diagnostic.log. *)
let target_name = "vc_mulhi_product_split_from_limbs"
let target_formula_hash = -633486862402357400

let expected_wrapper_axioms =
  [ "t'inj"; "extensionality"; "to_of_int"; "add_in_bounds";
    "add_wrapping"; "sub_in_bounds"; "sub_wrapping"; "mul_in_bounds";
    "mul_wrapping"; "neg_zero"; "neg_not_zero" ]

let in_wrapper_line_range line =
  (line >= 958 && line <= 992) || (line >= 1276 && line <= 1310)

let source_is_creusot_int file =
  let suffix = "/creusot/int.coma" in
  let lf = String.length file and ls = String.length suffix in
  lf >= ls && String.sub file (lf - ls) ls = suffix

let wrapper_axiom_info d =
  match d.Decl.d_node with
  | Decl.Dprop (Decl.Paxiom, pr, _) ->
      begin match pr.Decl.pr_name.Ident.id_loc with
      | Some loc ->
          let (file, line, _, _, _) = Loc.get loc in
          let name = pr.Decl.pr_name.Ident.id_string in
          if source_is_creusot_int file && in_wrapper_line_range line
             && List.mem name expected_wrapper_axioms
          then Some name
          else None
      | None -> None
      end
  | _ -> None

let sorted xs = List.sort String.compare xs

let expected_names = sorted (expected_wrapper_axioms @ expected_wrapper_axioms)

let task_matches task =
  let (goal_ident, _, _) = Termcode.goal_expl_task ~root:false task in
  let formula = Task.task_goal_fmla task in
  let names =
    List.filter_map wrapper_axiom_info (Task.task_decls task)
  in
  String.equal goal_ident.Ident.id_string target_name
  && Int.equal (Term.t_hash_strict formula) target_formula_hash
  && sorted names = expected_names

let count_wrappers task =
  List.length (List.filter_map wrapper_axiom_info (Task.task_decls task))

let filter_wrapper_axiom d =
  match wrapper_axiom_info d with
  | Some _ -> []
  | None -> [d]

let prune_if_exact_target task =
  if not (task_matches task) then task
  else begin
    let before_goal = Task.task_goal_fmla task in
    let before_decls = List.length (Task.task_decls task) in
    let before_wrappers = count_wrappers task in
    let transformed = Trans.apply (Trans.decl filter_wrapper_axiom None) task in
    let after_goal = Task.task_goal_fmla transformed in
    let after_decls = List.length (Task.task_decls transformed) in
    let after_wrappers = count_wrappers transformed in
    Format.eprintf
      "MULHI_PRUNE exact_target=true hash=%d declarations=%d->%d wrapper_axioms=%d->%d goal_unchanged=%b@."
      (Term.t_hash_strict before_goal) before_decls after_decls
      before_wrappers after_wrappers (Term.t_equal_strict before_goal after_goal);
    transformed
  end

let () =
  Trans.register_transform
    ~desc:"Drop only the two generated u128/u64 wrapper axiom families on the exact mulhi product-split VC"
    "mulhi_prune_product_leaf" (Trans.store prune_if_exact_target)
