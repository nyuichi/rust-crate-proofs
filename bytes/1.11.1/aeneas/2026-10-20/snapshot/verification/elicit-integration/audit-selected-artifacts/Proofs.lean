import ElicitContracts

namespace BytesElicit

theorem bytes_len_returns_stored_length : ElicitContracts.c_26b458661b6dffb1 := by
  intro self _
  exact ⟨self.len, rfl, rfl⟩

theorem bytes_mut_len_returns_stored_length : ElicitContracts.c_6aa70a75372247cf := by
  intro self _
  exact ⟨self.len, rfl, rfl⟩

theorem take_new_fields : ElicitContracts.c_03820710b16d0a9a := by
  intro T inner limit _
  exact ⟨{ inner := inner, limit := limit }, rfl, by simp⟩

theorem take_into_inner_returns_inner : ElicitContracts.c_67ab1aad488d2950 := by
  intro T self _
  exact ⟨self.inner, rfl, rfl⟩

theorem take_limit_returns_stored_limit : ElicitContracts.c_fe100f19c2bba7e6 := by
  intro T self _
  exact ⟨self.limit, rfl, rfl⟩

theorem take_set_limit_changes_only_limit : ElicitContracts.c_6664141dae95d86d := by
  intro T self lim _
  exact ⟨{ self with limit := lim }, rfl, by simp⟩

theorem chain_new_fields : ElicitContracts.c_76945f5528ea3028 := by
  intro T U a b _
  exact ⟨{ a := a, b := b }, rfl, by simp⟩

theorem chain_first_ref_returns_first : ElicitContracts.c_0a3770d6fb2dbe56 := by
  intro T U self _
  exact ⟨self.a, rfl, rfl⟩

theorem chain_first_mut_replaces_first : ElicitContracts.c_0ab59922312519d0 := by
  intro T U self _
  refine ⟨(self.a, fun replacement => { self with a := replacement }), rfl, ?_⟩
  simp

theorem chain_last_ref_returns_second : ElicitContracts.c_7142f49d9af652c8 := by
  intro T U self _
  exact ⟨self.b, rfl, rfl⟩

theorem chain_last_mut_replaces_second : ElicitContracts.c_4be354299d19c21d := by
  intro T U self _
  refine ⟨(self.b, fun replacement => { self with b := replacement }), rfl, ?_⟩
  simp

theorem chain_into_inner_returns_fields : ElicitContracts.c_cdf4d241f0e1fb55 := by
  intro T U self _
  exact ⟨(self.a, self.b), rfl, by simp⟩

theorem take_get_ref_returns_inner : ElicitContracts.c_f1abf57d300de3dd := by
  intro T self _
  exact ⟨self.inner, rfl, rfl⟩

theorem take_get_mut_replaces_inner_only : ElicitContracts.c_69283305e385fbc7 := by
  intro T self _
  refine ⟨(self.inner, fun replacement => { self with inner := replacement }), rfl, ?_⟩
  simp

theorem limit_new_fields : ElicitContracts.c_3c397f36f32e60b3 := by
  intro T inner limit _
  exact ⟨{ inner := inner, limit := limit }, rfl, by simp⟩

theorem limit_get_ref_returns_inner : ElicitContracts.c_5df6b3de38349faa := by
  intro T self _
  exact ⟨self.inner, rfl, rfl⟩

theorem limit_get_mut_replaces_inner_only : ElicitContracts.c_c3a35ebbb8331cde := by
  intro T self _
  refine ⟨(self.inner, fun replacement => { self with inner := replacement }), rfl, ?_⟩
  simp

theorem limit_returns_stored_limit : ElicitContracts.c_a904d034ecb38bd1 := by
  intro T self _
  exact ⟨self.limit, rfl, rfl⟩

theorem limit_set_limit_changes_only_limit : ElicitContracts.c_b9c9fc9a329282f1 := by
  intro T self lim _
  exact ⟨{ self with limit := lim }, rfl, by simp⟩

theorem limit_into_inner_returns_inner : ElicitContracts.c_6f39d01bffb2e600 := by
  intro T self _
  exact ⟨self.inner, rfl, rfl⟩

theorem into_iter_new_field : ElicitContracts.c_e3299c6f74df6cb4 := by
  intro T inner _
  exact ⟨{ inner := inner }, rfl, by simp⟩

theorem into_iter_get_ref_returns_inner : ElicitContracts.c_de579782e2a34b9b := by
  intro T self _
  exact ⟨self.inner, rfl, rfl⟩

theorem into_iter_get_mut_replaces_inner : ElicitContracts.c_a0e18b6702bd3796 := by
  intro T self _
  refine ⟨(self.inner, fun replacement => { self with inner := replacement }), rfl, ?_⟩
  simp

theorem into_iter_into_inner_returns_inner : ElicitContracts.c_0348592fe8519fce := by
  intro T self _
  exact ⟨self.inner, rfl, rfl⟩

end BytesElicit
