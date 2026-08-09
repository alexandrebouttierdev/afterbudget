# Remédiation de l'audit de production — Plan d'implémentation

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Corriger les 14 constats (AB-001 → AB-014) de l'audit de production d'AfterBudget, avec tests de non-régression, en gardant toutes les portes vertes.

**Architecture:** Corrections dans les modules existants (Vue → Message → DTO → validateur → commande → service → repository). `update.rs` cesse d'accéder aux repositories et à la copie de fichier ; les migrations deviennent transactionnelles avec une version v3 ; `Money` devient 100 % entier. TDD constat par constat, commit par constat.

**Tech Stack:** Rust 2021, Iced 0.13, rusqlite 0.31 (bundled/backup), chrono, uuid, validator 0.19+, tempfile (dev).

## Global Constraints

- Invariant utilisateur : un **solde** peut être négatif (onboarding, paramètres) ; le montant d'une **transaction/récurrence** est strictement positif (`valider_montant` rejette `<= 0`, `CHECK(amount_cents > 0)` en base). À verrouiller par tests à chaque étape concernée.
- Montants : centimes `i64` uniquement, jamais `f64` sur un chemin de montant (AB-005).
- Langue des messages et tests : français, style du code existant.
- Le binaire applique v1 → v2 → v3 ; les bases v1/v2 existantes doivent migrer sans intervention.
- `cargo audit` doit revenir à code 0 : `validator 0.19+` (idna 1.x) + exception `lru` documentée dans `.cargo/audit.toml`.
- Chaque migration est atomique (SQL + enregistrement de version dans une transaction) et idempotente.
- Les `ALTER TABLE ADD COLUMN` (non idempotents) ne sont jamais dans les fichiers SQL rejoués tels quels : garde `colonne_existe` dans `migrations.rs`.
- Commande de vérification rapide après chaque étape : `cargo test --lib` puis `cargo clippy --all-targets --all-features -- -D warnings` (sauf mention contraire).

---

### Task 1: AB-006 — Tronquage de note par frontière de caractère

**Files:**
- Modify: `src/core/utils.rs` (ajouter `tronquer_texte` + tests)
- Modify: `src/app/update.rs:374-377` (tronquage)
- Modify: `src/modules/transactions/validateurs.rs:34-41` (comptage chars)

**Interfaces:**
- Produces: `pub fn tronquer_texte(texte: &str, max_caracteres: usize) -> String` dans `core::utils`.

- [ ] **Step 1: Test rouge — helper de tronquage sûr**

Dans `src/core/utils.rs`, ajouter dans le module `#[cfg(test)] mod tests` :

```rust
/// La tronquage se fait par frontière de caractère : un caractère Unicode
/// multioctet ne doit jamais être coupé en deux (AB-006).
#[test]
fn un_texte_long_est_tronque_par_caractere() {
    assert_eq!(tronquer_texte("abcdef", 3), "abc");
    assert_eq!(tronquer_texte("a😀b😀c", 3), "a😀b");
    assert_eq!(tronquer_texte("a😀b😀c", 10), "a😀b😀c");
    assert_eq!(tronquer_texte("", 5), "");
    // 1 + 1001 emojis = 1002 caractères : le tronquage doit ramener à 1000.
    let emojis = "a".to_string() + &"😀".repeat(1001);
    assert_eq!(tronquer_texte(&emojis, 1000).chars().count(), 1000);
}
```

- [ ] **Step 2: Vérifier l'échec**

Run: `cargo test --lib core::utils::tests::un_texte_long_est_tronque_par_caractere`
Expected: FAIL (fonction `tronquer_texte` inconnue).

- [ ] **Step 3: Implémentation minimale**

Dans `src/core/utils.rs`, avant le module `tests` :

```rust
/// Tronque un texte à un nombre maximal de caractères, sans jamais couper un
/// caractère Unicode au milieu (AB-006). La limite du contrat est exprimée en
/// caractères, pas en octets.
pub fn tronquer_texte(texte: &str, max_caracteres: usize) -> String {
    texte.chars().take(max_caracteres).collect()
}
```

- [ ] **Step 4: Vérifier le succès**

Run: `cargo test --lib core::utils::tests::un_texte_long_est_tronque_par_caractere`
Expected: PASS.

- [ ] **Step 5: Brancher le tronquage dans update.rs**

Dans `src/app/update.rs`, remplacer les lignes 374-377 :

```rust
            let mut note_raw = state.transaction_form.note.clone();
            if note_raw.len() > 1000 {
                note_raw = note_raw[..1000].to_string();
            }
```

par :

```rust
            // Limite exprimée en caractères : le tronquage ne doit jamais
            // couper un caractère Unicode (AB-006).
            let note_raw =
                crate::core::utils::tronquer_texte(&state.transaction_form.note, 1000);
```

- [ ] **Step 6: Aligner le validateur sur les caractères**

Dans `src/modules/transactions/validateurs.rs`, remplacer le bloc note :

```rust
    if let Some(ref note) = dto.note {
        if note.len() > 1000 {
```

par :

```rust
    if let Some(ref note) = dto.note {
        if note.chars().count() > 1000 {
```

- [ ] **Step 7: Test de non-régression du validateur**

Dans `src/modules/transactions/validateurs.rs`, module tests, ajouter :

```rust
    /// Une note de 1000 caractères multioctets passe, 1001 est refusée :
    /// la limite compte des caractères, pas des octets (AB-006).
    #[test]
    fn la_limite_de_note_compte_les_caracteres() {
        let ok = {
            let mut d = dto_valide();
            d.note = Some("😀".repeat(1000));
            d
        };
        assert!(valider_creation(&ok).is_ok());

        let trop = {
            let mut d = dto_valide();
            d.note = Some("😀".repeat(1001));
            d
        };
        let erreurs = valider_creation(&trop).unwrap_err();
        assert!(erreurs.iter().any(|(champ, _)| champ == "note"));
    }
```

- [ ] **Step 8: Vérifier**

Run: `cargo test --lib` — Expected: PASS (tous).
Run: `cargo clippy --all-targets --all-features -- -D warnings` — Expected: code 0.

- [ ] **Step 9: Commit**

```bash
git add src/core/utils.rs src/app/update.rs src/modules/transactions/validateurs.rs
git commit -m "fix: tronquer la note par frontière de caractère (AB-006)"
```

---

### Task 2: AB-005 — Money : arithmétique entière sans f64

**Files:**
- Modify: `src/domaine/argent.rs`
- Modify: `src/domaine/budget.rs`
- Modify: `src/domaine/transaction.rs`
- Modify: `src/modules/statistiques/service.rs`
- Modify: `src/modules/budget/service.rs`
- Modify: `src/modules/recurrences/service.rs`
- Modify: `src/modules/transactions/service.rs`
- Modify: `src/app/update.rs` (`montant_editable`)
- Modify: `src/modules/transactions/views/index.rs` (appel `solde_des`)
- Modify: `src/modules/transactions/composants/ligne_transaction.rs:65` (aucun changement si `abs()` conservé)
- Test: `src/domaine/argent.rs` (module tests existant)

**Interfaces:**
- Consumes: `Money` tel que défini avant cette tâche.
- Produces:
  - `Money::from_input(s: &str) -> Result<Money, String>` — parsing manuel, plus de f64.
  - `Money::checked_add(self, Self) -> Option<Self>`, `checked_sub`, `checked_abs` — `None` sur débordement.
  - `Money::abs(self) -> Self` — `saturating_abs` (jamais de panic, pour l'affichage).
  - `Money::saturating_add(self, Self) -> Self`.
  - `BudgetSummary::compute(...) -> Result<BudgetSummary, String>`.
  - `Transaction::signed_amount() -> Result<Money, String>` ; `Transaction::apply_to_balance(&self, Money, Money) -> Result<Money, String>`.
  - `solde_des(&[Transaction]) -> Result<Money, String>` (service transactions).
  - `montant_editable(Money) -> String` sans f64.

- [ ] **Step 1: Tests rouges — parsing exact et bornes**

Dans `src/domaine/argent.rs`, remplacer le test `test_from_euros` par :

```rust
    #[test]
    fn test_parsing_entier_exact() {
        assert_eq!(Money::from_input("10").unwrap().cents, 1000);
        assert_eq!(Money::from_input("10,50").unwrap().cents, 1050);
        assert_eq!(Money::from_input("10.50").unwrap().cents, 1050);
        assert_eq!(Money::from_input("-478").unwrap().cents, -47800);
        assert_eq!(Money::from_input("1 207,50").unwrap().cents, 120750);
        assert_eq!(Money::from_input("1\u{00a0}207,50").unwrap().cents, 120750);
        assert_eq!(Money::from_input("1\u{202f}207,50").unwrap().cents, 120750);
    }
```

Et ajouter :

```rust
    /// Trois décimales sont refusées : pas d'arrondi silencieux (AB-005).
    #[test]
    fn trois_decimales_sont_refusees() {
        assert!(Money::from_input("12,345").is_err());
        assert!(Money::from_input("12.345").is_err());
    }

    /// Les bornes i64 sont les bornes du contrat : dépassement refusé, pas de
    /// wrap, pas de panic.
    #[test]
    fn les_bornes_i64_encadrent_le_montant() {
        let max = "9223372036854775,807";
        assert_eq!(
            Money::from_input(max).unwrap().cents,
            i64::MAX
        );
        let min = "-9223372036854775,808";
        assert_eq!(
            Money::from_input(min).unwrap().cents,
            i64::MIN
        );
        assert!(Money::from_input("9223372036854775,808").is_err());
        assert!(Money::from_input("-9223372036854775,809").is_err());
    }

    /// Un exposant scientifique n'est pas une saisie monétaire.
    #[test]
    fn une_saisie_scientifique_est_refusee() {
        assert!(Money::from_input("1e5").is_err());
    }

    #[test]
    fn laddition_verifiee_detecte_le_debordement() {
        let a = Money::from_cents(i64::MAX);
        assert_eq!(a.checked_add(Money::from_cents(1)), None);
        assert_eq!(a.checked_add(Money::ZERO), Some(a));
        let b = Money::from_cents(i64::MIN);
        assert_eq!(b.checked_sub(Money::from_cents(1)), None);
        assert_eq!(b.checked_abs(), None);
        assert_eq!(Money::from_cents(-5).checked_abs(), Some(Money::from_cents(5)));
    }

    /// L'affichage ne doit jamais paniquer, même sur la valeur minimale
    /// (l'abs littéral de i64::MIN est un débordement).
    #[test]
    fn laffichage_ne_panique_pas_sur_min() {
        let m = Money::from_cents(i64::MIN);
        let s = m.format_fr();
        assert!(s.contains(','));
        assert!(s.contains('€'));
    }
```

- [ ] **Step 2: Vérifier les échecs**

Run: `cargo test --lib domaine::argent`
Expected: FAIL — `from_euros` encore présent mais parsing via f64 : les tests bornes/3-décimales échouent (arrondi/acceptation), `checked_*` inconnus.

- [ ] **Step 3: Réécrire `argent.rs`**

Remplacer la section `impl Money` (lignes 9-153) par :

```rust
impl Money {
    pub const ZERO: Money = Money { cents: 0 };

    pub fn from_cents(cents: i64) -> Self {
        Self { cents }
    }

    /// Parse une saisie monétaire française en centimes, sans aucune
    /// conversion flottante (AB-005).
    ///
    /// Accepte un signe `-` en tête (un solde peut être négatif), des espaces
    /// de milliers (ordinaires ou insécables) et un unique séparateur décimal
    /// `,` ou `.`. Au-delà de deux décimales, la saisie est refusée — jamais
    /// arrondie. Le débordement de `i64` est une erreur.
    pub fn from_input(s: &str) -> Result<Self, String> {
        let cleaned = s
            .replace(['\u{00a0}', '\u{202f}', ' '], "")
            .replace(',', ".");

        let trimmed = cleaned.trim();
        if trimmed.is_empty() {
            return Err("Le montant est obligatoire.".into());
        }

        let (negatif, corps) = match trimmed.as_bytes()[0] {
            b'-' => (true, &trimmed[1..]),
            b'+' => (false, &trimmed[1..]),
            _ => (false, trimmed.as_str()),
        };
        if corps.is_empty() {
            return Err(format!("Montant invalide : « {} »", s));
        }

        let (entiers, decimales) = match corps.split_once('.') {
            Some((e, d)) => (e, d),
            None => (corps, ""),
        };
        if !entiers.chars().all(|c| c.is_ascii_digit())
            || !decimales.chars().all(|c| c.is_ascii_digit())
        {
            return Err(format!("Montant invalide : « {} »", s));
        }
        if decimales.len() > 2 {
            return Err(format!("Montant invalide : « {} »", s));
        }

        let entiers: i64 = entiers
            .parse()
            .map_err(|_| "Le montant est trop grand.".to_string())?;
        let decimales: i64 = if decimales.is_empty() {
            0
        } else if decimales.len() == 1 {
            decimales.parse::<i64>().map_err(|_| "Le montant est trop grand.".to_string())? * 10
        } else {
            decimales.parse().map_err(|_| "Le montant est trop grand.".to_string())?
        };

        let centimes = entiers
            .checked_mul(100)
            .and_then(|v| v.checked_add(decimales))
            .ok_or_else(|| "Le montant est trop grand.".to_string())?;
        let centimes = if negatif {
            centimes
                .checked_neg()
                .ok_or_else(|| "Le montant est trop grand.".to_string())?
        } else {
            centimes
        };

        Ok(Self { cents: centimes })
    }

    /// Filtre une frappe destinée à un champ montant.
    ///
    /// Ne laisse passer que les chiffres et un unique séparateur décimal ; les
    /// lettres et les signes sont simplement ignorés, ce qui évite d'afficher
    /// une erreur pour une touche que l'utilisateur n'aurait pas dû pouvoir
    /// saisir. Le séparateur est normalisé en virgule, usage français.
    pub fn filtrer_saisie(saisie: &str) -> String {
        let mut resultat = String::with_capacity(saisie.len());
        let mut separateur_place = false;
        let mut decimales = 0;

        for caractere in saisie.chars() {
            match caractere {
                '0'..='9' => {
                    if separateur_place {
                        // Au-delà de deux décimales, la frappe est ignorée.
                        if decimales == 2 {
                            continue;
                        }
                        decimales += 1;
                    }
                    resultat.push(caractere);
                }
                ',' | '.' if !separateur_place && !resultat.is_empty() => {
                    separateur_place = true;
                    resultat.push(',');
                }
                _ => {}
            }
        }

        resultat
    }

    /// Comme `filtrer_saisie`, mais conserve un signe moins en tête : un solde
    /// de compte peut être négatif, contrairement au montant d'une transaction.
    pub fn filtrer_saisie_signee(saisie: &str) -> String {
        let negatif = saisie.trim_start().starts_with('-');
        let chiffres = Self::filtrer_saisie(saisie);
        if negatif && !chiffres.is_empty() {
            format!("-{chiffres}")
        } else if negatif {
            "-".to_string()
        } else {
            chiffres
        }
    }

    pub fn format_fr(&self) -> String {
        // unsigned_abs : jamais de panic sur i64::MIN (AB-005).
        let abs_cents = self.cents.unsigned_abs();
        let euros_part = abs_cents / 100;
        let cents_part = abs_cents % 100;

        let mut result = String::new();
        if self.cents < 0 {
            result.push('-');
        }

        let euros_str = format!("{}", euros_part);
        let n = euros_str.len();
        for (i, c) in euros_str.chars().enumerate() {
            if i > 0 && (n - i) % 3 == 0 {
                result.push('\u{00a0}');
            }
            result.push(c);
        }

        result.push(',');
        result.push_str(&format!("{:02}", cents_part));
        result.push('\u{00a0}');
        result.push('€');

        result
    }

    pub fn format_fr_signed(&self) -> String {
        if self.cents >= 0 {
            format!("+{}", self.format_fr())
        } else {
            self.format_fr()
        }
    }

    pub fn is_zero(&self) -> bool {
        self.cents == 0
    }

    pub fn is_positive(&self) -> bool {
        self.cents > 0
    }

    pub fn is_negative(&self) -> bool {
        self.cents < 0
    }

    /// Valeur absolue pour l'affichage : saturée, jamais de panic (AB-005).
    pub fn abs(&self) -> Self {
        Self {
            cents: self.cents.saturating_abs(),
        }
    }

    pub fn checked_abs(self) -> Option<Self> {
        self.cents.checked_abs().map(|cents| Self { cents })
    }

    pub fn checked_add(self, other: Self) -> Option<Self> {
        self.cents.checked_add(other.cents).map(|cents| Self { cents })
    }

    pub fn checked_sub(self, other: Self) -> Option<Self> {
        self.cents.checked_sub(other.cents).map(|cents| Self { cents })
    }

    /// Addition saturée : utilisée uniquement pour des affichages de synthèse.
    pub fn saturating_add(self, other: Self) -> Self {
        Self {
            cents: self.cents.saturating_add(other.cents),
        }
    }
}
```

Puis **supprimer** les blocs `impl ops::Add`, `impl ops::AddAssign`, `impl ops::Sub`, `impl ops::SubAssign` (lignes 161-189) ainsi que `use std::ops;` (ligne 2) et la méthode `from_euros`.

- [ ] **Step 4: Vérifier les échecs de compilation des appelants**

Run: `cargo check --lib`
Expected: erreurs de compilation listant chaque appelant des opérateurs `+`/`-`, de `from_euros` et de `to_euros_f64`. Corriger dans les étapes suivantes.

- [ ] **Step 5: `domaine/transaction.rs`**

Remplacer `apply_to_balance` et `signed_amount` :

```rust
    pub fn apply_to_balance(&self, balance: Money, amount: Money) -> Result<Money, String> {
        match self {
            Self::Income => balance
                .checked_add(amount)
                .ok_or_else(|| "Dépassement de montant dans le calcul du solde.".into()),
            Self::Expense => balance
                .checked_sub(amount)
                .ok_or_else(|| "Dépassement de montant dans le calcul du solde.".into()),
        }
    }
```

```rust
    pub fn signed_amount(&self) -> Result<Money, String> {
        match self.kind {
            TransactionKind::Income => Ok(self.amount),
            TransactionKind::Expense => self
                .amount
                .cents
                .checked_neg()
                .map(Money::from_cents)
                .ok_or_else(|| "Dépassement de montant.".to_string()),
        }
    }
```

- [ ] **Step 6: `domaine/budget.rs` — compute vérifié**

Remplacer `pub fn compute(` (bloc lignes 52-88) par :

```rust
    pub fn compute(
        current_balance: Money,
        overdraft_limit: Money,
        pending_income: Money,
        pending_expenses: Money,
        total_income: Money,
        total_expenses: Money,
        completed_income: Money,
        completed_expenses: Money,
    ) -> Result<Self, String> {
        let err = || "Dépassement de montant dans le calcul du budget.".to_string();
        let projected_balance = current_balance
            .checked_add(pending_income)
            .ok_or_else(err)?
            .checked_sub(pending_expenses)
            .ok_or_else(err)?;
        let remaining_overdraft_margin = projected_balance
            .checked_add(overdraft_limit)
            .ok_or_else(err)?;

        let financial_status = if projected_balance >= Money::ZERO {
            FinancialStatus::Healthy
        } else if projected_balance >= Money::from_cents(-overdraft_limit.cents) {
            FinancialStatus::Warning
        } else {
            FinancialStatus::Danger
        };

        Ok(Self {
            current_balance,
            pending_income,
            pending_expenses,
            projected_balance,
            overdraft_limit,
            remaining_overdraft_margin,
            financial_status,
            total_income,
            total_expenses,
            completed_income,
            completed_expenses,
        })
    }
```

Dans le même fichier, remplacer les trois helpers (lignes 98-118) :

```rust
    pub fn decouvert_restant(&self) -> Money {
        if self.projected_balance.is_negative() {
            Money::from_cents(
                self.overdraft_limit
                    .cents
                    .saturating_add(self.projected_balance.cents)
                    .max(0),
            )
        } else {
            self.overdraft_limit
        }
    }

    pub fn decouvert_utilise(&self) -> Money {
        if self.projected_balance.is_negative() {
            Money::from_cents(
                (-self.projected_balance.cents).min(self.overdraft_limit.cents),
            )
        } else {
            Money::ZERO
        }
    }

    pub fn depassement_du_decouvert(&self) -> Money {
        Money::from_cents((-self.remaining_overdraft_margin.cents).max(0))
    }
```

- [ ] **Step 7: `budget.rs` module tests — adapter aux nouvelles signatures**

Dans `src/domaine/budget.rs`, module `tests` : remplacer **chaque** appel `BudgetSummary::compute(...)` par la même expression suffixée de `.unwrap()` (le résultat devient `Result`). Remplacer chaque `Money::from_euros(X.0)` par `Money::from_cents(X00)` selon le tableau :

| ancien | nouveau |
|---|---|
| `from_euros(300.0)` | `from_cents(30000)` |
| `from_euros(200.0)` | `from_cents(20000)` |
| `from_euros(1500.0)` | `from_cents(150000)` |
| `from_euros(1200.0)` | `from_cents(120000)` |
| `from_euros(0.0)` | `ZERO` |
| `from_euros(-360.0)` | `from_cents(-36000)` |
| `from_euros(500.0)` | `from_cents(50000)` |
| `from_euros(1207.0)` | `from_cents(120700)` |
| `from_euros(1325.0)` | `from_cents(132500)` |
| `from_euros(-400.0)` | `from_cents(-40000)` |
| `from_euros(100.0)` | `from_cents(10000)` |

Le test `test_exclusion_transactions_realisees` (lignes 296-308) devient :

```rust
    #[test]
    fn test_exclusion_transactions_realisees() {
        let summary = BudgetSummary::compute(
            Money::from_cents(10000),
            Money::from_cents(10000),
            Money::ZERO,
            Money::ZERO,
            Money::from_cents(50000),
            Money::from_cents(30000),
            Money::from_cents(50000),
            Money::from_cents(30000),
        )
        .unwrap();
        assert_eq!(summary.projected_balance.cents, 10000);
    }
```

- [ ] **Step 8: `modules/budget/service.rs` — propagation**

Remplacer les lignes 20-21 :

```rust
    let completed_income = total_income - pending_income;
    let completed_expenses = total_expenses - pending_expenses;
```

par :

```rust
    let completed_income = total_income
        .checked_sub(pending_income)
        .ok_or_else(|| "Dépassement de montant dans le calcul du budget.".to_string())?;
    let completed_expenses = total_expenses
        .checked_sub(pending_expenses)
        .ok_or_else(|| "Dépassement de montant dans le calcul du budget.".to_string())?;
```

Et la fin de `calculer_resume_budget` (lignes 23-32) :

```rust
    Ok(BudgetSummary::compute(
        settings.current_balance,
        settings.overdraft_limit,
        pending_income,
        pending_expenses,
        total_income,
        total_expenses,
        completed_income,
        completed_expenses,
    )?)
```

- [ ] **Step 9: `modules/budget/tests/mod.rs` — adapter**

Remplacer dans `setup_scenario` (lignes 13-18) :

```rust
    let settings = AppSettings {
        current_balance: Money::from_cents(-36000),
        overdraft_limit: Money::from_cents(50000),
        onboarding_completed: true,
        ..Default::default()
    };
```

Remplacer les deux appels `budget_service::calculer_resume_budget(&pool, 2026, 8)` (lignes 73 et ailleurs) — la signature ne change pas, seul `BudgetSummary::compute` interne change ; vérifier qu'aucun `from_euros` ne subsiste dans le fichier.

- [ ] **Step 10: `modules/statistiques/service.rs` — propagation**

Remplacer les lignes 22-25 :

```rust
    let completed_income = total_income - pending_income;
    let completed_expenses = total_expenses - pending_expenses;

    let balance = total_income - total_expenses;
```

par :

```rust
    let completed_income = total_income
        .checked_sub(pending_income)
        .ok_or_else(|| "Dépassement de montant dans les statistiques.".to_string())?;
    let completed_expenses = total_expenses
        .checked_sub(pending_expenses)
        .ok_or_else(|| "Dépassement de montant dans les statistiques.".to_string())?;

    let balance = total_income
        .checked_sub(total_expenses)
        .ok_or_else(|| "Dépassement de montant dans les statistiques.".to_string())?;
```

- [ ] **Step 11: `modules/recurrences/service.rs` — `total_mensuel` saturé**

Remplacer (lignes 84-88) :

```rust
pub fn total_mensuel(regles: &[RecurringRule], genre: TransactionKind) -> Money {
    regles
        .iter()
        .filter(|r| r.is_active && r.kind == genre)
        .fold(Money::ZERO, |total, r| total.saturating_add(r.amount))
}
```

- [ ] **Step 12: `modules/transactions/service.rs` — `solde_des` vérifié**

Remplacer (lignes 90-93) :

```rust
/// Solde signé d'un ensemble de transactions : revenus moins dépenses.
///
/// Utilisé pour afficher le total des lignes réellement visibles après
/// filtrage, ce qui donne un sens immédiat au filtre appliqué.
pub fn solde_des(transactions: &[Transaction]) -> Result<Money, String> {
    let mut total = Money::ZERO;
    for transaction in transactions {
        let signe = transaction.signed_amount()?;
        total = total
            .checked_add(signe)
            .ok_or_else(|| "Dépassement de montant dans le total des lignes.".to_string())?;
    }
    Ok(total)
}
```

- [ ] **Step 13: `app/update.rs` — `montant_editable` sans f64**

Remplacer (lignes 739-743) :

```rust
/// Met un montant sous une forme directement réutilisable dans un champ de
/// saisie : sans séparateur de milliers ni symbole, virgule décimale.
pub fn montant_editable(montant: Money) -> String {
    let signe = if montant.cents < 0 { "-" } else { "" };
    let abs = montant.cents.unsigned_abs();
    format!("{}{},{}", signe, abs / 100, format!("{:02}", abs % 100))
}
```

- [ ] **Step 14: `modules/transactions/views/index.rs:95` — adapter `solde_des`**

Lire la ligne 95 et adapter l'appel :

```rust
    let total = crate::modules::transactions::service::solde_des(&state.transactions);
```

en :

```rust
    let total = crate::modules::transactions::service::solde_des(&state.transactions)
        .unwrap_or(crate::domaine::argent::Money::ZERO);
```

(adapter selon le code réellement présent, garder la valeur de repli `Money::ZERO`).

- [ ] **Step 15: Vérification complète**

Run: `cargo test --lib` — Expected: PASS.
Run: `cargo clippy --all-targets --all-features -- -D warnings` — Expected: code 0.
Run: `grep -rn "from_euros\|to_euros_f64" src/` — Expected: aucune occurrence.

- [ ] **Step 16: Commit**

```bash
git add -A src/
git commit -m "fix: arithmétique monétaire entière sans f64 (AB-005)"
```

---

### Task 3: AB-001/AB-009 — Lignes affectées vérifiées dans les repositories

**Files:**
- Modify: `src/modules/transactions/repository.rs` (`update`, `delete_by_id`)
- Modify: `src/modules/categories/repository.rs` (`update`)
- Modify: `src/modules/recurrences/repository.rs` (`supprimer`)
- Modify: `src/modules/parametres/repository.rs` (`update_settings`)
- Test: `src/modules/transactions/tests/mod.rs`, `src/modules/recurrences/tests.rs`

**Interfaces:**
- Consumes: aucune nouvelle.
- Produces: les `UPDATE`/`DELETE` échouent avec `Err` quand `rows_affected == 0`.

- [ ] **Step 1: Tests rouges — suppression/modification d'une ligne absente**

Dans `src/modules/transactions/tests/mod.rs`, ajouter :

```rust
    /// Modifier ou supprimer une transaction inexistante doit échouer : un
    /// succès silencieux ferait croire à une persistance qui n'a pas eu lieu
    /// (AB-001, AB-009).
    #[test]
    fn modifier_une_transaction_absente_echoue() {
        let pool = base_temporaire::creer_base_test();
        let tx = donnees_test::transaction_test(
            "Fantôme",
            1000,
            TransactionKind::Expense,
            TransactionStatus::Pending,
            "2026-08-01",
            "autre",
        );
        assert!(repo::update(&pool, &tx).is_err());
        assert!(repo::delete_by_id(&pool, "id-inexistant").is_err());
    }
```

Dans `src/modules/recurrences/tests.rs`, ajouter :

```rust
/// Supprimer une règle inexistante doit échouer (AB-001, AB-009).
#[test]
fn supprimer_une_regle_absente_echoue() {
    let pool = base();
    assert!(commandes::supprimer(&pool, "id-inexistant").is_err());
}
```

- [ ] **Step 2: Vérifier les échecs**

Run: `cargo test --lib modules::transactions::tests::modifier_une_transaction_absente_echoue modules::recurrences::tests::supprimer_une_regle_absente_echoue`
Expected: FAIL (les fonctions retournent `Ok(())`).

- [ ] **Step 3: Implémenter — transactions**

Dans `src/modules/transactions/repository.rs`, `update` (lignes 273-294), remplacer la fin :

```rust
        .map_err(|e| format!("Erreur de mise à jour : {}", e))?;
    Ok(())
}
```

par :

```rust
        .map_err(|e| format!("Erreur de mise à jour : {}", e))?;

    if maj == 0 {
        return Err(format!("Transaction introuvable : {}", t.id));
    }
    Ok(())
}
```

(et convertir la chaîne en variable : `let maj = pool.conn.execute(...)`).

Même traitement pour `delete_by_id` (lignes 296-301) :

```rust
pub fn delete_by_id(pool: &DatabasePool, id: &str) -> Result<(), String> {
    let supprimes = pool
        .conn
        .execute("DELETE FROM transactions WHERE id = ?1", params![id])
        .map_err(|e| format!("Erreur de suppression : {}", e))?;

    if supprimes == 0 {
        return Err(format!("Transaction introuvable : {}", id));
    }
    Ok(())
}
```

- [ ] **Step 4: Implémenter — catégories**

Dans `src/modules/categories/repository.rs`, `update` (lignes 193-211) :

```rust
pub fn update(pool: &DatabasePool, cat: &Category) -> Result<(), String> {
    let now = chrono::Utc::now().to_rfc3339();
    let maj = pool
        .conn
        .execute(
            "UPDATE categories SET name = ?1, icon = ?2, color = ?3, sort_order = ?4, is_active = ?5, updated_at = ?6
             WHERE id = ?7",
            params![
                cat.name,
                cat.icon,
                cat.color,
                cat.sort_order,
                cat.is_active as i32,
                &now,
                cat.id,
            ],
        )
        .map_err(|e| format!("Erreur de mise à jour : {}", e))?;

    if maj == 0 {
        return Err(format!("Catégorie introuvable : {}", cat.id));
    }
    Ok(())
}
```

- [ ] **Step 5: Implémenter — récurrences**

Dans `src/modules/recurrences/repository.rs`, `supprimer` (lignes 89-105) :

```rust
pub fn supprimer(pool: &DatabasePool, identifiant: &str) -> Result<(), String> {
    pool.conn
        .execute(
            "UPDATE transactions SET recurring_rule_id = NULL WHERE recurring_rule_id = ?1",
            params![identifiant],
        )
        .map_err(|e| format!("Détachement des transactions : {e}"))?;
    let supprimees = pool
        .conn
        .execute(
            "DELETE FROM recurring_rules WHERE id = ?1",
            params![identifiant],
        )
        .map_err(|e| format!("Suppression récurrence : {e}"))?;

    if supprimees == 0 {
        return Err(format!("Récurrence introuvable : {identifiant}"));
    }
    Ok(())
}
```

- [ ] **Step 6: Implémenter — paramètres**

Dans `src/modules/parametres/repository.rs`, `update_settings` (lignes 58-79) :

```rust
pub fn update_settings(pool: &DatabasePool, s: &AppSettings) -> Result<(), String> {
    let now = chrono::Utc::now().to_rfc3339();
    let maj = pool
        .conn
        .execute(
            "UPDATE app_settings SET current_balance_cents = ?1, overdraft_limit_cents = ?2,
             currency_code = ?3, locale = ?4, theme = ?5, balance_updated_at = ?6,
             onboarding_completed = ?7, updated_at = ?8
             WHERE id = 1",
            params![
                s.current_balance.cents,
                s.overdraft_limit.cents,
                s.currency_code,
                s.locale,
                s.theme,
                s.balance_updated_at,
                s.onboarding_completed as i32,
                &now,
            ],
        )
        .map_err(|e| format!("Erreur de mise à jour des paramètres : {}", e))?;

    if maj == 0 {
        return Err("Aucun paramètre à mettre à jour : la ligne id=1 est absente.".into());
    }
    Ok(())
}
```

- [ ] **Step 7: Vérifier**

Run: `cargo test --lib` — Expected: PASS (tous).
Run: `cargo clippy --all-targets --all-features -- -D warnings` — Expected: code 0.

- [ ] **Step 8: Commit**

```bash
git add src/modules/
git commit -m "fix: vérifier les lignes affectées par les écritures (AB-001, AB-009)"
```

---

### Task 4: AB-001 — Propagation des erreurs dans `update.rs`

**Files:**
- Modify: `src/app/update.rs` (handlers de mutation, `appliquer_theme`, `CompleteOnboarding`, `SubmitTransactionForm`, `DeleteRecurringRule`)
- Test: `src/modules/parametres/tests/mod.rs` (nouveau) + `#[cfg(test)] mod tests;` dans `src/modules/parametres/mod.rs`

**Interfaces:**
- Consumes: services `parametres_service::*` (retournent `Result`), `tx_cmd::*`.
- Produces: plus aucun `let _ =` ni `.ok()` sur un chemin d'écriture dans `update.rs` ; notifications d'erreur en cas d'échec d'écriture ou de rechargement.

- [ ] **Step 1: Tests rouges — persistance sous contrainte SQLite**

Créer `src/modules/parametres/tests/mod.rs` :

```rust
use crate::core::db::pool::DatabasePool;
use crate::domaine::argent::Money;
use crate::modules::parametres::service;

mod commun;
use commun::base_temporaire;

/// Une connexion non inscriptible ne doit pas produire un faux succès : la
/// mise à jour échoue proprement et la valeur en base reste inchangée
/// (AB-001).
#[test]
fn une_connexion_non_inscriptible_echoue_sans_effet() {
    let pool = base_temporaire::creer_base_test();
    let initial = service::obtenir_parametres(&pool).unwrap();
    let _ = initial; // pas encore de ligne : insérons-en une.

    use crate::domaine::parametres::AppSettings;
    let parametres = AppSettings {
        current_balance: Money::from_cents(10000),
        ..Default::default()
    };
    crate::modules::parametres::repository::insert_settings(&pool, &parametres).unwrap();

    pool.conn
        .execute_batch("PRAGMA query_only = ON;")
        .expect("pragma");

    let erreur = service::mettre_a_jour_solde(&pool, Money::from_cents(5000));
    assert!(erreur.is_err(), "l'écriture devait échouer");

    pool.conn
        .execute_batch("PRAGMA query_only = OFF;")
        .expect("pragma");
    let relu = service::obtenir_parametres(&pool).unwrap().unwrap();
    assert_eq!(relu.current_balance.cents, 10000, "la valeur doit rester inchangée");
}

/// Mettre à jour les paramètres d'une base sans ligne id=1 échoue (AB-001).
#[test]
fn mettre_a_jour_sans_ligne_de_parametres_echoue() {
    let pool = base_temporaire::creer_base_test();
    let mut parametres = crate::domaine::parametres::AppSettings::default();
    parametres.current_balance = Money::from_cents(1);
    assert!(
        crate::modules::parametres::repository::update_settings(&pool, &parametres).is_err()
    );
}
```

Créer `src/modules/parametres/tests/commun.rs` :

```rust
pub use crate::tests_commun::*;
```

Dans `src/modules/parametres/mod.rs`, ajouter en fin de fichier :

```rust
#[cfg(test)]
mod tests;
```

- [ ] **Step 2: Vérifier l'état des tests**

Run: `cargo test --lib modules::parametres`
Expected : `mettre_a_jour_sans_ligne_de_parametres_echoue` FAIL (rows_affected non vérifié — corrigé en Task 3, donc déjà vert ici si Task 3 a été exécutée) ; `une_connexion_non_inscriptible_echoue_sans_effet` doit déjà passer (rusqlite refuse l'écriture sous `query_only`). Les deux sont les tests de non-régression AB-001 : noter le résultat et continuer.

- [ ] **Step 3: Réécrire les handlers de mutation dans `update.rs`**

Remplacer `Message::UpdateBalance` (lignes 120-140) :

```rust
        Message::UpdateBalance => {
            let balance = match Money::from_input(&state.settings_balance_str) {
                Ok(m) => m,
                Err(e) => {
                    state.settings_error = Some(e);
                    return Task::none();
                }
            };

            let Some(db) = state.db.as_ref() else {
                return Task::none();
            };
            if let Err(e) = parametres_service::mettre_a_jour_solde(db, balance) {
                state.notification =
                    Some(Notification::erreur(format!("Solde non enregistré : {e}")));
                return Task::none();
            }
            state.settings = parametres_service::obtenir_parametres(db).ok().flatten();

            if let Err(e) = state.load_month_data() {
                state.notification = Some(Notification::erreur(format!(
                    "Solde enregistré, mais rechargement impossible : {e}"
                )));
                return Task::none();
            }

            state.notification = Some(Notification::succes("Solde mis à jour."));
            state.settings_balance_str.clear();
            state.edition_solde_ouverte = false;
            Task::none()
        }
```

Remplacer `Message::UpdateOverdraft` (lignes 146-170) :

```rust
        Message::UpdateOverdraft => {
            let overdraft = match Money::from_input(&state.settings_overdraft_str) {
                Ok(m) => {
                    if m.cents < 0 {
                        state.settings_error =
                            Some("Le découvert ne peut pas être négatif.".into());
                        return Task::none();
                    }
                    m
                }
                Err(e) => {
                    state.settings_error = Some(e);
                    return Task::none();
                }
            };

            let Some(db) = state.db.as_ref() else {
                return Task::none();
            };
            if let Err(e) = parametres_service::mettre_a_jour_decouvert(db, overdraft) {
                state.notification =
                    Some(Notification::erreur(format!("Découvert non enregistré : {e}")));
                return Task::none();
            }
            state.settings = parametres_service::obtenir_parametres(db).ok().flatten();

            if let Err(e) = state.load_month_data() {
                state.notification = Some(Notification::erreur(format!(
                    "Découvert enregistré, mais rechargement impossible : {e}"
                )));
                return Task::none();
            }

            state.notification = Some(Notification::succes("Découvert mis à jour."));
            state.settings_overdraft_str.clear();
            Task::none()
        }
```

Remplacer `Message::SetCurrency` (lignes 179-186) :

```rust
        Message::SetCurrency(cur) => {
            let Some(db) = state.db.as_ref() else {
                return Task::none();
            };
            if let Err(e) = parametres_service::mettre_a_jour_devise(db, &cur) {
                state.notification =
                    Some(Notification::erreur(format!("Devise non enregistrée : {e}")));
                return Task::none();
            }
            state.settings = parametres_service::obtenir_parametres(db).ok().flatten();

            if let Err(e) = state.load_month_data() {
                state.notification = Some(Notification::erreur(format!(
                    "Devise enregistrée, mais rechargement impossible : {e}"
                )));
                return Task::none();
            }

            state.notification = Some(Notification::succes(format!("Devise changée pour {cur}.")));
            Task::none()
        }
```

Remplacer `Message::CompleteOnboarding` (lignes 76-96) :

```rust
        Message::CompleteOnboarding => {
            let dto = TerminerOnboardingDto {
                solde_actuel: state.onboarding_balance_str.clone(),
                decouvert_autorise: state.onboarding_overdraft_str.clone(),
                devise: state.onboarding_currency.clone(),
            };

            let Some(db) = state.db.as_ref() else {
                return Task::none();
            };
            match onboarding_cmd::terminer(db, &dto) {
                Ok(()) => match state.load_data() {
                    Ok(()) => {
                        state.notification =
                            Some(Notification::succes("Bienvenue dans AfterBudget !"));
                    }
                    Err(e) => {
                        state.notification = Some(Notification::erreur(format!(
                            "Paramètres enregistrés, mais rechargement impossible : {e}"
                        )));
                    }
                },
                Err(e) => {
                    state.onboarding_error = Some(e);
                }
            }
            Task::none()
        }
```

- [ ] **Step 4: `ConfirmDeleteTransaction`, `ToggleTransactionStatus`, `DeleteRecurringRule`**

Remplacer `Message::ConfirmDeleteTransaction` (lignes 480-491) :

```rust
        Message::ConfirmDeleteTransaction => {
            let Some(tx) = state.delete_transaction.clone() else {
                return Task::none();
            };
            let Some(db) = state.db.as_ref() else {
                return Task::none();
            };

            if let Err(e) = tx_cmd::supprimer(db, &tx.id) {
                state.notification =
                    Some(Notification::erreur(format!("Suppression impossible : {e}")));
                return Task::none();
            }
            state.delete_transaction = None;
            state.show_delete_confirm = false;

            if let Err(e) = state.load_month_data() {
                state.notification = Some(Notification::erreur(format!(
                    "Suppression enregistrée, mais rechargement impossible : {e}"
                )));
                return Task::none();
            }
            state.notification = Some(Notification::succes("Transaction supprimée."));
            Task::none()
        }
```

Remplacer `Message::ToggleTransactionStatus` (lignes 497-504) :

```rust
        Message::ToggleTransactionStatus(id) => {
            let Some(db) = state.db.as_ref() else {
                return Task::none();
            };
            if let Err(e) = tx_cmd::changer_statut(db, &id) {
                state.notification =
                    Some(Notification::erreur(format!("Statut non modifié : {e}")));
                return Task::none();
            }
            if let Err(e) = state.load_month_data() {
                state.notification = Some(Notification::erreur(format!(
                    "Statut modifié, mais rechargement impossible : {e}"
                )));
                return Task::none();
            }
            state.notification = Some(Notification::succes("Statut modifié."));
            Task::none()
        }
```

Remplacer `Message::DeleteRecurringRule` (lignes 335-352) :

```rust
        Message::DeleteRecurringRule(identifiant) => {
            let Some(db) = state.db.as_ref() else {
                return Task::none();
            };
            match recurrences_cmd::supprimer(db, &identifiant) {
                Ok(()) => {
                    if let Err(e) = state.load_data() {
                        state.notification = Some(Notification::erreur(format!(
                            "Récurrence supprimée, mais rechargement impossible : {e}"
                        )));
                        return Task::none();
                    }
                    state.notification = Some(Notification::succes(
                        "Récurrence supprimée. Les occurrences déjà créées sont conservées.",
                    ));
                }
                Err(e) => {
                    state.notification = Some(Notification::erreur(format!(
                        "Suppression impossible : {e}"
                    )));
                }
            }
            Task::none()
        }
```

- [ ] **Step 5: `SubmitTransactionForm` — succès après rechargement**

Remplacer le bras `Ok(())` du `match result` (lignes 447-460) :

```rust
                Ok(()) => {
                    state.show_transaction_form = false;
                    state.transaction_form = TransactionFormState::default();
                    if let Err(e) = state.load_month_data() {
                        state.notification = Some(Notification::erreur(format!(
                            "Transaction enregistrée, mais rechargement impossible : {e}"
                        )));
                        return Task::none();
                    }
                    let msg = if was_edit {
                        "Transaction modifiée."
                    } else {
                        match form_kind {
                            TransactionKind::Income => "Revenu ajouté.",
                            TransactionKind::Expense => "Dépense ajoutée.",
                        }
                    };
                    state.notification = Some(Notification::succes(msg));
                }
```

Et le bras récurrence `Ok(regle)` (lignes 403-412) :

```rust
                    Ok(regle) => {
                        state.show_transaction_form = false;
                        state.transaction_form = TransactionFormState::default();
                        if let Err(e) = state.load_data() {
                            state.notification = Some(Notification::erreur(format!(
                                "Récurrence créée, mais rechargement impossible : {e}"
                            )));
                            return Task::none();
                        }
                        state.notification = Some(Notification::succes(format!(
                            "Récurrence créée : {}.",
                            regle.periodicite().to_lowercase()
                        )));
                        Task::none()
                    }
```

- [ ] **Step 6: `appliquer_theme`**

Remplacer (lignes 745-752) :

```rust
/// Applique et persiste un mode de thème.
fn appliquer_theme(state: &mut AppState, mode: ThemeMode) -> Task<Message> {
    state.theme_mode = mode;
    let Some(db) = state.db.as_ref() else {
        return Task::none();
    };
    if let Err(e) = parametres_service::mettre_a_jour_theme(db, mode.cle()) {
        state.notification =
            Some(Notification::erreur(format!("Thème non enregistré : {e}")));
    } else {
        state.settings = parametres_service::obtenir_parametres(db).ok().flatten();
    }
    Task::none()
}
```

- [ ] **Step 7: ExportDatabase — mémorisation du dernier export vérifiée**

Remplacer le bloc (lignes 552-558) :

```rust
                    Ok(()) => {
                        if let Ok(Some(mut s)) = params_repo::get_settings(db) {
                            s.last_export_date = Some(chrono::Utc::now().to_rfc3339());
                            let _ = params_repo::update_settings(db, &s);
                            state.settings = Some(s);
                        }
```

par :

```rust
                    Ok(()) => {
                        if let Some(mut s) = params_repo::get_settings(db)
                            .map_err(|e| format!("Relecture des paramètres impossible : {e}"))?
                        {
                            s.last_export_date = Some(chrono::Utc::now().to_rfc3339());
                            params_repo::update_settings(db, &s)?;
                            state.settings = Some(s);
                        }
```

- [ ] **Step 8: Vérifier**

Run: `cargo test --lib` — Expected: PASS.
Run: `cargo clippy --all-targets --all-features -- -D warnings` — Expected: code 0.
Run: `grep -n "let _ =\|\.ok()" src/app/update.rs` — Expected : il ne reste que les `.ok()` des **lectures** de navigation (loads) ; aucune sur un chemin d'écriture.

- [ ] **Step 9: Commit**

```bash
git add src/app/update.rs src/modules/parametres/
git commit -m "fix: ne notifier le succès qu'après écriture et rechargement confirmés (AB-001)"
```

---

### Task 5: AB-008 — Mappers stricts sans valeurs par défaut

**Files:**
- Modify: `src/modules/transactions/repository.rs`
- Modify: `src/modules/categories/repository.rs`
- Modify: `src/modules/recurrences/repository.rs`
- Test: `src/modules/transactions/tests/mod.rs`, `src/modules/categories/tests/mod.rs`, `src/modules/recurrences/tests.rs`

**Interfaces:**
- Produces: `row_to_transaction(&TransactionRow) -> Result<Transaction, String>`, `row_to_category(&CategoryRow) -> Result<Category, String>`, `depuis_ligne(...) -> rusqlite::Result<RecurringRule>` (erreur sur données invalides, avec l'identifiant de ligne).

- [ ] **Step 1: Tests rouges — ligne invalide refuse le chargement**

Dans `src/modules/transactions/tests/mod.rs`, ajouter :

```rust
    /// Une ligne SQLite invalide doit faire échouer le chargement avec un
    /// message contextualisé, jamais être transformée en valeurs par défaut
    /// (AB-008).
    #[test]
    fn une_ligne_invalide_fait_echouer_le_chargement() {
        let pool = base_temporaire::creer_base_test();
        // Les CHECK du schéma bloquent déjà kind/statut invalides à l'insertion ;
        // la date et les horodatages, eux, passent : c'est sur eux que le
        // mapping strict doit échouer (AB-008).
        pool.conn
            .execute(
                "INSERT INTO transactions (id, kind, label, amount_cents, transaction_date, status,
                    category_id, note, created_at, updated_at)
                 VALUES ('ligne-corrompue', 'expense', 'X', 1000, '2026-99-99', 'pending',
                    'autre', NULL, 'pas-un-horodatage', '2026-08-01T00:00:00Z')",
                [],
            )
            .unwrap();

        let erreur = repo::find_all_by_month(&pool, 2026, 8).unwrap_err();
        assert!(
            erreur.contains("ligne-corrompue"),
            "l'erreur doit nommer la ligne : {erreur}"
        );
    }
```

Dans `src/modules/categories/tests/mod.rs`, ajouter :

```rust
    /// Une catégorie aux horodatages invalides fait échouer le chargement
    /// (AB-008). Le kind invalide est bloqué par le CHECK du schéma.
    #[test]
    fn une_categorie_invalide_fait_echouer_le_chargement() {
        let pool = base_temporaire::creer_base_test();
        pool.conn
            .execute(
                "INSERT INTO categories (id, kind, name, icon, color, sort_order, is_default, is_active, created_at, updated_at)
                 VALUES ('cat-corrompue', 'expense', 'X', 'x', '#3B82F6', 99, 0, 1, 'pas-un-horodatage', '2026-08-01T00:00:00Z')",
                [],
            )
            .unwrap();
        assert!(repo::find_all_active(&pool).is_err());
        assert!(repo::find_by_id(&pool, "cat-corrompue").is_err());
    }
```

- [ ] **Step 2: Vérifier les échecs**

Run: `cargo test --lib modules::transactions::tests::une_ligne_invalide_fait_echouer_le_chargement modules::categories::tests::une_categorie_invalide_fait_echouer_le_chargement`
Expected: FAIL (chargement « réussit » avec valeurs par défaut).

Ajouter aussi dans `src/modules/recurrences/tests.rs` :

```rust
/// Une règle aux horodatages invalides fait échouer le listage (AB-008).
#[test]
fn une_regle_corrompue_fait_echouer_le_listage() {
    let pool = base();
    pool.conn
        .execute(
            "INSERT INTO recurring_rules (id, kind, label, amount_cents, category_id,
                day_of_month, start_year, start_month, end_year, end_month, note,
                is_active, created_at, updated_at)
             VALUES ('regle-corrompue', 'expense', 'X', 1000, 'logement', 5, 2026, 8,
                NULL, NULL, NULL, 1, 'pas-un-horodatage', '2026-08-01T00:00:00Z')",
            [],
        )
        .unwrap();
    assert!(commandes::lister(&pool).is_err());
}
```

- [ ] **Step 3: Implémenter — transactions**

Dans `src/modules/transactions/repository.rs`, remplacer `row_to_transaction` (lignes 8-30) :

```rust
fn row_to_transaction(row: &TransactionRow) -> Result<Transaction, String> {
    use crate::domaine::argent::Money;
    use crate::domaine::transaction::{TransactionKind, TransactionStatus};

    let kind = TransactionKind::from_str(&row.kind).ok_or_else(|| {
        format!("Transaction {} : type inconnu « {} ».", row.id, row.kind)
    })?;
    let transaction_date = chrono::NaiveDate::parse_from_str(&row.transaction_date, "%Y-%m-%d")
        .map_err(|_| {
            format!(
                "Transaction {} : date invalide « {} ».",
                row.id, row.transaction_date
            )
        })?;
    let status = TransactionStatus::from_str(&row.status).ok_or_else(|| {
        format!("Transaction {} : statut inconnu « {} ».", row.id, row.status)
    })?;
    let parse_horodatage = |brut: &str| {
        chrono::DateTime::parse_from_rfc3339(brut)
            .map(|d| d.with_timezone(&chrono::Utc))
            .map_err(|_| {
                format!(
                    "Transaction {} : horodatage invalide « {} ».",
                    row.id, brut
                )
            })
    };

    Ok(Transaction {
        id: row.id.clone(),
        kind,
        label: row.label.clone(),
        amount: Money::from_cents(row.amount_cents),
        transaction_date,
        status,
        category_id: row.category_id.clone(),
        note: row.note.clone(),
        recurring_rule_id: row.recurring_rule_id.clone(),
        created_at: parse_horodatage(&row.created_at)?,
        updated_at: parse_horodatage(&row.updated_at)?,
    })
}
```

Puis remplacer les trois sites d'utilisation :

`find_by_month` (lignes 112-116) :

```rust
    let mut transactions = Vec::new();
    for row in rows {
        let row = row.map_err(|e| format!("Erreur de lecture : {}", e))?;
        transactions.push(row_to_transaction(&row)?);
    }
```

`find_recent_by_month` (lignes 168-172) : même remplacement.

`find_by_id` (lignes 204-208) :

```rust
    match rows.next() {
        Some(Ok(row)) => Ok(Some(row_to_transaction(&row)?)),
        Some(Err(e)) => Err(format!("Erreur de lecture : {}", e)),
        None => Ok(None),
    }
```

- [ ] **Step 4: Implémenter — catégories**

Dans `src/modules/categories/repository.rs`, remplacer `row_to_category` (lignes 8-25) :

```rust
fn row_to_category(row: &CategoryRow) -> Result<Category, String> {
    let kind = crate::domaine::transaction::TransactionKind::from_str(&row.kind).ok_or_else(|| {
        format!("Catégorie {} : type inconnu « {} ».", row.id, row.kind)
    })?;
    let parse_horodatage = |brut: &str| {
        chrono::DateTime::parse_from_rfc3339(brut)
            .map(|d| d.with_timezone(&chrono::Utc))
            .map_err(|_| format!("Catégorie {} : horodatage invalide « {} ».", row.id, brut))
    };

    Ok(Category {
        id: row.id.clone(),
        kind,
        name: row.name.clone(),
        icon: row.icon.clone(),
        color: row.color.clone(),
        sort_order: row.sort_order,
        is_default: row.is_default,
        is_active: row.is_active,
        created_at: parse_horodatage(&row.created_at)?,
        updated_at: parse_horodatage(&row.updated_at)?,
    })
}
```

Adapter les trois sites : `find_all_active`, `find_all`, `find_by_kind` (boucles `for row in rows { ... push(row_to_category(&row)?); }`) et `find_by_id` :

```rust
    match rows.next() {
        Some(Ok(row)) => Ok(Some(row_to_category(&row)?)),
        Some(Err(e)) => Err(format!("Erreur de lecture : {}", e)),
        None => Ok(None),
    }
```

- [ ] **Step 5: Implémenter — récurrences**

Dans `src/modules/recurrences/repository.rs`, remplacer `depuis_ligne` (lignes 11-41) :

```rust
fn depuis_ligne(ligne: &rusqlite::Row<'_>) -> rusqlite::Result<RecurringRule> {
    let id: String = ligne.get(0)?;
    let genre: String = ligne.get(1)?;
    let fin_annee: Option<i32> = ligne.get(8)?;
    let fin_mois: Option<u32> = ligne.get(9)?;
    let cree: String = ligne.get(12)?;
    let modifie: String = ligne.get(13)?;

    let kind = TransactionKind::from_str(&genre)
        .ok_or_else(|| rusqlite::Error::FromSqlConversionFailure(
            1,
            rusqlite::types::Type::Text,
            format!("Récurrence {id} : type inconnu « {genre} ».").into(),
        ))?;

    let horodatage = |brut: &str| {
        chrono::DateTime::parse_from_rfc3339(brut).map(|d| d.with_timezone(&chrono::Utc))
    };

    Ok(RecurringRule {
        id,
        kind,
        label: ligne.get(2)?,
        amount: Money::from_cents(ligne.get(3)?),
        category_id: ligne.get(4)?,
        day_of_month: ligne.get(5)?,
        start: (ligne.get(6)?, ligne.get(7)?),
        end: match (fin_annee, fin_mois) {
            (Some(annee), Some(mois)) => Some((annee, mois)),
            _ => None,
        },
        note: ligne.get(10)?,
        is_active: ligne.get::<_, i64>(11)? != 0,
        created_at: horodatage(&cree).ok_or_else(|| {
            rusqlite::Error::FromSqlConversionFailure(
                12,
                rusqlite::types::Type::Text,
                format!("Récurrence {id} : horodatage invalide « {cree} ».").into(),
            )
        })?,
        updated_at: horodatage(&modifie).ok_or_else(|| {
            rusqlite::Error::FromSqlConversionFailure(
                13,
                rusqlite::types::Type::Text,
                format!("Récurrence {id} : horodatage invalide « {modifie} ».").into(),
            )
        })?,
    })
}
```

- [ ] **Step 6: Vérifier**

Run: `cargo test --lib` — Expected: PASS (les deux nouveaux tests passent ; les 220+ tests existants aussi).
Run: `cargo clippy --all-targets --all-features -- -D warnings` — Expected: code 0.

- [ ] **Step 7: Commit**

```bash
git add src/modules/
git commit -m "fix: mapping strict des lignes SQLite, plus de valeurs par défaut silencieuses (AB-008)"
```

---

### Task 6: AB-009 — Invariants catégorie / type / activité

**Files:**
- Modify: `src/modules/categories/service.rs` (helper `verifier_categorie`)
- Modify: `src/modules/transactions/service.rs` (appel)
- Modify: `src/modules/recurrences/service.rs` (appel)
- Test: `src/modules/transactions/tests/mod.rs`

**Interfaces:**
- Consumes: `categories::repository::find_by_id`, `TransactionKind`.
- Produces: `categories::service::verifier_categorie(pool, kind, category_id) -> Result<(), String>`.

- [ ] **Step 1: Test rouge — matrice catégorie × type**

Dans `src/modules/transactions/tests/mod.rs`, ajouter :

```rust
    /// Une transaction ne peut référencer qu'une catégorie existante, active
    /// et du même sens (AB-009).
    #[test]
    fn une_transaction_verifie_la_categorie() {
        let pool = base_temporaire::creer_base_test();

        // Catégorie income désactivée et catégorie expense active.
        let maintenant = chrono::Utc::now().to_rfc3339();
        pool.conn
            .execute(
                "INSERT INTO categories (id, kind, name, icon, color, sort_order, is_default, is_active, created_at, updated_at)
                 VALUES ('income-inactive', 'income', 'Bonus inactif', 'x', '#3B82F6', 99, 0, 0, ?1, ?1)",
                rusqlite::params![&maintenant],
            )
            .unwrap();

        use crate::modules::transactions::service;
        use crate::domaine::argent::Money;

        assert!(service::creer_transaction(
            &pool,
            TransactionKind::Expense,
            "Courses",
            Money::from_cents(5000),
            chrono::NaiveDate::from_ymd_opt(2026, 8, 1).unwrap(),
            "alimentation",
            TransactionStatus::Pending,
            None,
        )
        .is_ok());

        // Catégorie inconnue.
        assert!(service::creer_transaction(
            &pool,
            TransactionKind::Expense,
            "Courses",
            Money::from_cents(5000),
            chrono::NaiveDate::from_ymd_opt(2026, 8, 1).unwrap(),
            "n-existe-pas",
            TransactionStatus::Pending,
            None,
        )
        .is_err());

        // Catégorie inactive.
        assert!(service::creer_transaction(
            &pool,
            TransactionKind::Income,
            "Bonus",
            Money::from_cents(5000),
            chrono::NaiveDate::from_ymd_opt(2026, 8, 1).unwrap(),
            "income-inactive",
            TransactionStatus::Pending,
            None,
        )
        .is_err());

        // Catégorie du mauvais sens.
        assert!(service::creer_transaction(
            &pool,
            TransactionKind::Expense,
            "Bonus",
            Money::from_cents(5000),
            chrono::NaiveDate::from_ymd_opt(2026, 8, 1).unwrap(),
            "salaire",
            TransactionStatus::Pending,
            None,
        )
        .is_err());

        // Rien d'autre n'a été persisté.
        let lignes = repo::find_all_by_month(&pool, 2026, 8).unwrap();
        assert_eq!(lignes.len(), 1);
    }
```

- [ ] **Step 2: Vérifier l'échec**

Run: `cargo test --lib modules::transactions::tests::une_transaction_verifie_la_categorie`
Expected: FAIL — la création sur catégorie inactive/inconnue réussit.

- [ ] **Step 3: Implémenter le helper**

Dans `src/modules/categories/service.rs`, ajouter :

```rust
use crate::domaine::transaction::TransactionKind;

/// Vérifie qu'une catégorie peut porter une transaction ou une règle du sens
/// donné : existante, active, et du même type (AB-009).
pub fn verifier_categorie(
    pool: &DatabasePool,
    kind: TransactionKind,
    category_id: &str,
) -> Result<(), String> {
    let cat = crate::modules::categories::repository::find_by_id(pool, category_id)?
        .ok_or_else(|| format!("Catégorie inconnue : « {} ».", category_id))?;
    if !cat.is_active {
        return Err(format!("La catégorie « {} » est désactivée.", cat.name));
    }
    if cat.kind != kind {
        return Err(format!(
            "La catégorie « {} » est une catégorie de {}, pas de {}.",
            cat.name,
            cat.kind.display_name(),
            kind.display_name()
        ));
    }
    Ok(())
}
```

- [ ] **Step 4: Brancher — transactions**

Dans `src/modules/transactions/service.rs`, en tête de `creer_transaction` (après les paramètres, avant la construction de `Transaction`) :

```rust
    crate::modules::categories::service::verifier_categorie(pool, kind, category_id)?;
```

et en tête de `modifier_transaction` :

```rust
pub fn modifier_transaction(pool: &DatabasePool, tx: &Transaction) -> Result<(), String> {
    crate::modules::categories::service::verifier_categorie(pool, tx.kind, &tx.category_id)?;
    repo::update(pool, tx)
}
```

- [ ] **Step 5: Brancher — récurrences**

Dans `src/modules/recurrences/service.rs`, dans `creer`, avant la construction de `RecurringRule` :

```rust
    crate::modules::categories::service::verifier_categorie(pool, genre, &dto.categorie_id)?;
```

- [ ] **Step 6: Vérifier**

Run: `cargo test --lib` — Expected: PASS.
Run: `cargo clippy --all-targets --all-features -- -D warnings` — Expected: code 0.
Attention : `src/modules/recurrences/tests.rs` crée des règles avec `categorie_id: "logement"` et `"salaire"` — catégories par défaut actives : PASS.

- [ ] **Step 7: Commit**

```bash
git add src/modules/
git commit -m "fix: vérifier existence, activité et type de la catégorie (AB-009)"
```

---

### Task 7: AB-007 — Migrations transactionnelles et v3

**Files:**
- Modify: `migrations/0002_recurrences.sql` (retirer l'ALTER)
- Create: `migrations/0003_last_export_date.sql`
- Modify: `src/core/db/migrations.rs`
- Modify: `tests/migrations.rs`
- Test: `tests/migrations.rs`

**Interfaces:**
- Produces: `pub const VERSION_COURANTE: i64` (= 3) ; `run_migrations(&Connection) -> Result<(), String>` ; chaque migration appliquée dans une transaction unique avec son enregistrement de version ; `colonne_existe(tx, table, colonne) -> Result<bool, String>`.

- [ ] **Step 1: Adapter le SQL v2 — retirer l'ALTER**

Dans `migrations/0002_recurrences.sql`, supprimer les lignes 24-26 (le bloc commentaire « Rattachement… » + l'`ALTER TABLE transactions ADD COLUMN recurring_rule_id …;`). La garde est désormais dans `migrations.rs`.

- [ ] **Step 2: Créer la migration v3**

Créer `migrations/0003_last_export_date.sql` :

```sql
-- Date du dernier export réussi.
--
-- Ajoutée par la migration v3. L'ALTER est rejoué uniquement si la colonne
-- n'existe pas encore (garde dans core/db/migrations.rs) : ce SQL n'est donc
-- jamais exécuté deux fois sur une base déjà à jour.
ALTER TABLE app_settings ADD COLUMN last_export_date TEXT;
```

- [ ] **Step 3: Tests rouges — transactionnalité et base interrompue**

Dans `tests/migrations.rs`, ajouter :

```rust
#[test]
fn une_base_interrompue_entre_alter_et_version_est_reparable() {
    let pool = DatabasePool::open_in_memory().unwrap();
    // v1 appliquée, colonne recurring_rule_id ajoutée par un v2 « à moitié
    // appliqué » (ALTER réussi, version jamais enregistrée) : le scénario
    // exact du bug AB-007.
    pool.conn
        .execute_batch(include_str!("../migrations/0001_initial.sql"))
        .unwrap();
    pool.conn
        .execute(
            "INSERT INTO schema_migrations VALUES (1, 'v1', ?1)",
            rusqlite::params![chrono::Utc::now().to_rfc3339()],
        )
        .unwrap();
    pool.conn
        .execute_batch(
            "ALTER TABLE transactions ADD COLUMN recurring_rule_id TEXT REFERENCES recurring_rules(id);",
        )
        .unwrap();

    assert!(migrations::run_migrations(&pool.conn).is_ok());

    let version: i64 = pool
        .conn
        .query_row("SELECT MAX(version) FROM schema_migrations", [], |r| r.get(0))
        .unwrap();
    assert_eq!(version, migrations::VERSION_COURANTE);

    let colonnes: i64 = pool
        .conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('transactions') WHERE name = 'recurring_rule_id'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(colonnes, 1);
}

#[test]
fn une_migration_qui_echoue_ne_laisse_rien_derriere() {
    // Test unitaire du mécanisme transactionnel, dans le module migrations.
}
```

- [ ] **Step 4: Test unitaire du mécanisme — dans `migrations.rs`**

Ajouter en fin de `src/core/db/migrations.rs` :

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn connexion() -> rusqlite::Connection {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                version INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                applied_at TEXT NOT NULL
            );",
        )
        .unwrap();
        conn
    }

    /// Une migration dont le SQL échoue doit laisser la base intacte : ni
    /// table partielle, ni version enregistrée (AB-007).
    #[test]
    fn une_migration_qui_echoue_est_annulee() {
        let conn = connexion();
        let echouante: MigrationFn = Box::new(|tx| {
            tx.execute_batch("CREATE TABLE partielle_test (x INTEGER);")?;
            Err("échec injecté".into())
        });
        assert!(appliquer_migration(&conn, 1, "v1", &echouante).is_err());

        let tables: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='partielle_test'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(tables, 0, "la table partielle doit être annulée");

        let version: i64 = conn
            .query_row("SELECT COALESCE(MAX(version), 0) FROM schema_migrations", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(version, 0, "aucune version ne doit être enregistrée");
    }

    /// Une migration réussie enregistre SQL et version ensemble.
    #[test]
    fn une_migration_reussie_enregistre_sql_et_version() {
        let conn = connexion();
        let reussie: MigrationFn = Box::new(|tx| {
            tx.execute_batch("CREATE TABLE complete_test (x INTEGER);")?;
            Ok(())
        });
        appliquer_migration(&conn, 1, "v1", &reussie).unwrap();

        let tables: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='complete_test'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(tables, 1);
        let version: i64 = conn
            .query_row("SELECT MAX(version) FROM schema_migrations", [], |r| r.get(0))
            .unwrap();
        assert_eq!(version, 1);
    }
}
```

- [ ] **Step 5: Vérifier les échecs**

Run: `cargo test --test migrations`
Expected: FAIL — `appliquer_migration` inconnue, `VERSION_COURANTE` inconnue.

- [ ] **Step 6: Réécrire `migrations.rs`**

Remplacer le fichier (lignes 1-80) :

```rust
use crate::domaine::categorie::{DEFAULT_EXPENSE_CATEGORIES, DEFAULT_INCOME_CATEGORIES};
use rusqlite::params;

/// Dernière version de schéma produite par cette application. Le validateur
/// d'import s'en sert pour accepter ou refuser un fichier (AB-002).
pub const VERSION_COURANTE: i64 = 3;

type MigrationFn = Box<dyn Fn(&rusqlite::Transaction) -> Result<(), String>>;

pub fn run_migrations(conn: &rusqlite::Connection) -> Result<(), String> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            applied_at TEXT NOT NULL
        );",
    )
    .map_err(|e| format!("Erreur schema_migrations : {}", e))?;

    let current = get_current_version(conn)?;
    let migrations: Vec<(i64, &str, MigrationFn)> = vec![
        (1, "v1", Box::new(migration_v1)),
        (2, "v2", Box::new(migration_v2)),
        (3, "v3", Box::new(migration_v3)),
    ];

    for (v, nom, m) in migrations {
        if v > current {
            appliquer_migration(conn, v, nom, &m)?;
        }
    }
    Ok(())
}

/// Applique une migration dans une transaction unique : le SQL et
/// l'enregistrement de version sont engagés ensemble, ou annulés ensemble
/// (AB-007).
fn appliquer_migration(
    conn: &rusqlite::Connection,
    v: i64,
    nom: &str,
    m: &MigrationFn,
) -> Result<(), String> {
    let tx = conn
        .unchecked_transaction()
        .map_err(|e| format!("Démarrage de la transaction v{v} : {e}"))?;
    m(&tx).map_err(|e| format!("Erreur migration {nom} (v{v}) : {e}"))?;

    let now = chrono::Utc::now().to_rfc3339();
    tx.execute(
        "INSERT INTO schema_migrations VALUES (?1, ?2, ?3)",
        params![v, nom, now],
    )
    .map_err(|e| format!("Erreur enregistrement migration v{v} : {e}"))?;

    tx.commit().map_err(|e| format!("Erreur commit migration v{v} : {e}"))?;
    Ok(())
}

fn get_current_version(conn: &rusqlite::Connection) -> Result<i64, String> {
    conn.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
        [],
        |r| r.get(0),
    )
    .map_err(|e| format!("Erreur version : {}", e))
}

fn migration_v1(tx: &rusqlite::Transaction) -> Result<(), String> {
    tx.execute_batch(include_str!("../../../migrations/0001_initial.sql"))
        .map_err(|e| format!("Erreur migration v1 : {}", e))?;
    insert_default_categories(tx)
}

fn migration_v2(tx: &rusqlite::Transaction) -> Result<(), String> {
    tx.execute_batch(include_str!("../../../migrations/0002_recurrences.sql"))
        .map_err(|e| format!("Erreur migration v2 : {}", e))?;

    // Garde d'idempotence : une base interrompue par l'ancien bug AB-007 a
    // déjà la colonne sans la version enregistrée. Ne jamais rejouer l'ALTER.
    if !colonne_existe(tx, "transactions", "recurring_rule_id")? {
        tx.execute_batch(
            "ALTER TABLE transactions ADD COLUMN recurring_rule_id TEXT REFERENCES recurring_rules(id);",
        )
        .map_err(|e| format!("Erreur migration v2 (colonne) : {}", e))?;
    }
    Ok(())
}

fn migration_v3(tx: &rusqlite::Transaction) -> Result<(), String> {
    if !colonne_existe(tx, "app_settings", "last_export_date")? {
        tx.execute_batch(include_str!("../../../migrations/0003_last_export_date.sql"))
            .map_err(|e| format!("Erreur migration v3 : {}", e))?;
    }
    Ok(())
}

/// Vrai si une colonne existe déjà dans une table : sert à rendre les ALTER
/// idempotents (AB-007).
fn colonne_existe(tx: &rusqlite::Transaction, table: &str, colonne: &str) -> Result<bool, String> {
    let mut stmt = tx
        .prepare(&format!("PRAGMA table_info({table})"))
        .map_err(|e| format!("Lecture du schéma de {table} : {e}"))?;
    let lignes = stmt
        .query_map([], |ligne| ligne.get::<_, String>(1))
        .map_err(|e| format!("Lecture du schéma de {table} : {e}"))?;
    for nom in lignes {
        let nom = nom.map_err(|e| format!("Lecture du schéma de {table} : {e}"))?;
        if nom == colonne {
            return Ok(true);
        }
    }
    Ok(false)
}

fn insert_default_categories(tx: &rusqlite::Transaction) -> Result<(), String> {
    let now = chrono::Utc::now().to_rfc3339();
    let all: Vec<(&str, &str, &str, &str, &str)> = DEFAULT_EXPENSE_CATEGORIES
        .iter()
        .map(|c| (c.id, "expense", c.name, c.icon, c.color))
        .chain(
            DEFAULT_INCOME_CATEGORIES
                .iter()
                .map(|c| (c.id, "income", c.name, c.icon, c.color)),
        )
        .collect();

    for (i, (id, kind, name, icon, color)) in all.iter().enumerate() {
        tx.execute(
            "INSERT OR IGNORE INTO categories (id, kind, name, icon, color, sort_order, is_default, is_active, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1, 1, ?7, ?7)",
            params![id, kind, name, icon, color, i as i32, &now],
        )
        .map_err(|e| format!("Insertion catégorie {} : {}", id, e))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    // (contenu de l'étape 4)
}
```

- [ ] **Step 7: Vérifier**

Run: `cargo test --test migrations` — Expected: PASS (6 tests).
Run: `cargo test --lib` — Expected: PASS.
Run: `cargo clippy --all-targets --all-features -- -D warnings` — Expected: code 0.

- [ ] **Step 8: Commit**

```bash
git add migrations/ src/core/db/migrations.rs tests/migrations.rs
git commit -m "fix: migrations transactionnelles et idempotentes, version v3 (AB-007)"
```

---

### Task 8: AB-002/AB-003 + AB-011 import — validation en lecture seule et remplacement atomique

**Files:**
- Modify: `src/core/db/pool.rs`
- Modify: `src/modules/import_export/service.rs`
- Modify: `src/modules/import_export/tests/mod.rs`
- Modify: `src/app/message.rs` (variant `ImportResult`)
- Modify: `src/app/update.rs` (`ConfirmImport`, suppression de `perform_import`)

**Interfaces:**
- Consumes: `migrations::VERSION_COURANTE`.
- Produces:
  - `DatabasePool::open_read_only(path) -> Result<Self, String>`
  - `DatabasePool::verifier_integrite(path) -> Result<(), String>`
  - `import_export::service::importer_en_arriere_plan(chemin_actuel: PathBuf, source: PathBuf) -> Result<(), String>`
  - `Message::ImportResult(Result<(), String>)`.

- [ ] **Step 1: Tests rouges — validation et remplacement**

Remplacer `src/modules/import_export/tests/mod.rs` :

```rust
//! Tests d'intégration pour le module import_export (AB-002, AB-003).

use crate::core::db::migrations;
use crate::core::db::pool::DatabasePool;
use crate::modules::import_export::service;

fn ecrire_base_v1(chemin: &std::path::Path) {
    let conn = rusqlite::Connection::open(chemin).unwrap();
    conn.execute_batch(include_str!("../../../migrations/0001_initial.sql"))
        .unwrap();
    conn.execute(
        "INSERT INTO schema_migrations VALUES (1, 'v1', ?1)",
        rusqlite::params![chrono::Utc::now().to_rfc3339()],
    )
    .unwrap();
}

fn ecrire_base_v2(chemin: &std::path::Path) {
    ecrire_base_v1(chemin);
    let conn = rusqlite::Connection::open(chemin).unwrap();
    conn.execute_batch(include_str!("../../../migrations/0002_recurrences.sql"))
        .unwrap();
    conn.execute_batch(
        "ALTER TABLE transactions ADD COLUMN recurring_rule_id TEXT REFERENCES recurring_rules(id);",
    )
    .unwrap();
    conn.execute(
        "INSERT INTO schema_migrations VALUES (2, 'v2', ?1)",
        rusqlite::params![chrono::Utc::now().to_rfc3339()],
    )
    .unwrap();
}

fn somme_checksum(chemin: &std::path::Path) -> u64 {
    std::fs::read(chemin)
        .unwrap()
        .iter()
        .fold(0u64, |acc, octet| acc.wrapping_add(*octet as u64))
}

#[test]
fn une_base_v1_et_une_base_v2_sont_acceptees() {
    let repertoire = tempfile::tempdir().unwrap();

    let v1 = repertoire.path().join("v1.sqlite");
    ecrire_base_v1(&v1);
    assert!(service::valider_import(&v1).is_ok(), "v1 doit être acceptée");

    let v2 = repertoire.path().join("v2.sqlite");
    ecrire_base_v2(&v2);
    assert!(service::valider_import(&v2).is_ok(), "v2 doit être acceptée");
}

#[test]
fn une_version_future_est_refusee() {
    let repertoire = tempfile::tempdir().unwrap();
    let chemin = repertoire.path().join("future.sqlite");
    ecrire_base_v1(&chemin);
    let conn = rusqlite::Connection::open(&chemin).unwrap();
    conn.execute(
        "INSERT INTO schema_migrations VALUES (99, 'v99', ?1)",
        rusqlite::params![chrono::Utc::now().to_rfc3339()],
    )
    .unwrap();
    drop(conn);

    let erreur = service::valider_import(&chemin).unwrap_err();
    assert!(erreur.contains("99"), "message attendu : {erreur}");
}

#[test]
fn un_fichier_non_sqlite_est_refuse() {
    let repertoire = tempfile::tempdir().unwrap();
    let chemin = repertoire.path().join("faux.sqlite");
    std::fs::write(&chemin, "pas une base du tout, juste du texte.").unwrap();
    assert!(service::valider_import(&chemin).is_err());
}

/// Un import réussi remplace la base ; un import échoué laisse la base
/// d'origine intacte (AB-003).
#[test]
fn un_import_reussi_remplace_et_un_import_echoue_preserve() {
    let repertoire = tempfile::tempdir().unwrap();

    // Base courante avec un paramètre.
    let cible = repertoire.path().join("cible.sqlite");
    {
        let pool = DatabasePool::open(&cible).unwrap();
        migrations::run_migrations(&pool.conn).unwrap();
        let mut parametres = crate::domaine::parametres::AppSettings::default();
        parametres.current_balance = crate::domaine::argent::Money::from_cents(11111);
        crate::modules::parametres::repository::insert_settings(&pool, &parametres).unwrap();
    }

    // Source valide avec une autre valeur.
    let source = repertoire.path().join("source.sqlite");
    {
        let pool = DatabasePool::open(&source).unwrap();
        migrations::run_migrations(&pool.conn).unwrap();
        let mut parametres = crate::domaine::parametres::AppSettings::default();
        parametres.current_balance = crate::domaine::argent::Money::from_cents(22222);
        crate::modules::parametres::repository::insert_settings(&pool, &parametres).unwrap();
    }

    service::importer_en_arriere_plan(cible.clone(), source.clone()).unwrap();
    let lue = crate::modules::parametres::repository::get_settings(
        &DatabasePool::open(&cible).unwrap(),
    )
    .unwrap()
    .unwrap();
    assert_eq!(lue.current_balance.cents, 22222, "les données importées doivent remplacer");

    // Échec : source corrompue (fichier tronqué). Le checksum de référence est
    // capturé APRÈS l'import réussi, sur l'état que l'échec ne doit pas toucher.
    let corrompue = repertoire.path().join("corrompue.sqlite");
    std::fs::write(&corrompue, b"SQLite format 3\0").unwrap();
    let avant = somme_checksum(&cible);
    assert!(service::importer_en_arriere_plan(cible.clone(), corrompue).is_err());
    let apres = somme_checksum(&cible);
    assert_eq!(avant, apres, "la base d'origine doit rester intacte");
}
```

- [ ] **Step 2: Vérifier les échecs**

Run: `cargo test --lib modules::import_export`
Expected: FAIL — fonctions manquantes.

- [ ] **Step 3: `pool.rs` — lecture seule, en-tête sans lecture complète, intégrité**

Remplacer la méthode `validate_sqlite_file` (lignes 60-70) :

```rust
    /// Vérifie qu'un fichier est une base SQLite valide : seule l'en-tête de
    /// 16 octets est lue, jamais le fichier entier (AB-003, AB-011).
    pub fn validate_sqlite_file(path: &Path) -> Result<(), String> {
        use std::io::Read;

        if !path.exists() {
            return Err("Fichier introuvable.".into());
        }
        let mut fichier =
            std::fs::File::open(path).map_err(|e| format!("Lecture impossible : {}", e))?;
        let mut en_tete = [0u8; 16];
        let lus = fichier
            .read(&mut en_tete)
            .map_err(|e| format!("Lecture impossible : {}", e))?;
        if lus < 16 || &en_tete != b"SQLite format 3\0" {
            return Err("Le fichier n'est pas une base SQLite.".into());
        }
        Ok(())
    }

    /// Ouvre une base en lecture seule : aucune écriture, aucun WAL/SHM créé.
    /// Sert à valider un fichier importé sans le toucher (AB-003).
    pub fn open_read_only(path: &Path) -> Result<Self, String> {
        let conn = rusqlite::Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(|e| format!("Ouverture en lecture seule impossible : {}", e))?;
        Ok(Self {
            conn,
            path: path.to_path_buf(),
        })
    }

    /// Exécute `PRAGMA integrity_check` sur un fichier, en lecture seule
    /// (AB-003).
    pub fn verifier_integrite(path: &Path) -> Result<(), String> {
        let pool = Self::open_read_only(path)?;
        let resultat: String = pool
            .conn
            .query_row("PRAGMA integrity_check", [], |ligne| ligne.get(0))
            .map_err(|e| format!("Vérification d'intégrité impossible : {}", e))?;
        if resultat != "ok" {
            return Err(format!("Base corrompue : {}", resultat));
        }
        Ok(())
    }
```

- [ ] **Step 4: `import_export/service.rs` — validation et import sûr**

Remplacer `valider_import` (lignes 9-59) :

```rust
pub fn valider_import(path: &Path) -> Result<(), String> {
    DatabasePool::validate_sqlite_file(path)?;
    DatabasePool::verifier_integrite(path)?;

    let temp = DatabasePool::open_read_only(path)?;

    let tables: Vec<String> = {
        let mut stmt = temp
            .conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table'")
            .map_err(|_| "Base de données illisible.".to_string())?;

        let rows = stmt
            .query_map([], |row| row.get(0))
            .map_err(|_| "Base de données illisible.".to_string())?;

        rows.filter_map(|r| r.ok()).collect()
    };

    let version: i64 = temp
        .conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let tables_requises: &[&str] = if version >= 2 {
        &[
            "transactions",
            "categories",
            "app_settings",
            "schema_migrations",
            "recurring_rules",
        ]
    } else {
        &["transactions", "categories", "app_settings", "schema_migrations"]
    };
    for table in tables_requises {
        if !tables.iter().any(|t| t == table) {
            return Err(format!(
                "Le fichier n'est pas une base AfterBudget valide (table « {} » manquante).",
                table
            ));
        }
    }

    if version < 1 {
        return Err("La base importée ne contient aucune migration enregistrée.".into());
    }
    if version > migrations::VERSION_COURANTE {
        return Err(format!(
            "La base importée utilise une version de schéma ({}) plus récente que celle supportée ({}).",
            version,
            migrations::VERSION_COURANTE
        ));
    }

    Ok(())
}
```

Remplacer `importer` (lignes 61-81) par la fonction de fond + helpers :

```rust
/// Importe un fichier validé dans la base courante, **hors du thread UI**
/// (AB-003, AB-011).
///
/// La séquence est : validation en lecture seule → backup → copie vers un
/// fichier temporaire du même répertoire → fsync → rename atomique → migration
/// du fichier importé. Tout échec après le backup restaure le backup ; si la
/// restauration échoue elle-même, le chemin de la sauvegarde est indiqué.
pub fn importer_en_arriere_plan(chemin_actuel: PathBuf, source: PathBuf) -> Result<(), String> {
    valider_import(&source)?;

    let backup_path = chemin_actuel
        .parent()
        .ok_or("Chemin de base invalide.")?
        .join(format!(
            "afterbudget-pre-import-{}.sqlite",
            chrono::Utc::now().format("%Y%m%d-%H%M%S")
        ));

    let pool = DatabasePool::open(&chemin_actuel)?;
    pool.backup_to(&backup_path)?;
    drop(pool);

    if let Err(e) = remplacer_fichier(&chemin_actuel, &source) {
        let restauration = restaurer_fichier(&chemin_actuel, &backup_path);
        return Err(match restauration {
            Ok(()) => format!("{e} ; tes données ont été restaurées."),
            Err(erreur) => format!(
                "{e} ; la restauration a échoué ({erreur}). Sauvegarde disponible : {}",
                backup_path.display()
            ),
        });
    }

    // Les anciens exports (v1/v2) doivent être migrés avant réouverture.
    let migre = DatabasePool::open(&chemin_actuel).and_then(|p| {
        migrations::run_migrations(&p.conn).map_err(|e| format!("Migration de l'import impossible : {e}"))
    });
    if let Err(e) = migre {
        let restauration = restaurer_fichier(&chemin_actuel, &backup_path);
        return Err(match restauration {
            Ok(()) => format!("{e} ; tes données ont été restaurées."),
            Err(erreur) => format!(
                "{e} ; la restauration a échoué ({erreur}). Sauvegarde disponible : {}",
                backup_path.display()
            ),
        });
    }

    Ok(())
}

/// Remplace `destination` par `source` de façon atomique : copie vers un
/// fichier temporaire du même répertoire, synchronisation, puis rename.
fn remplacer_fichier(destination: &Path, source: &Path) -> Result<(), String> {
    let repertoire = destination.parent().ok_or("Chemin de base invalide.")?;
    let temporaire = repertoire.join(format!(
        ".afterbudget-import-{}.sqlite",
        uuid::Uuid::new_v4()
    ));

    std::fs::copy(source, &temporaire)
        .map_err(|e| format!("Copie du fichier impossible : {e}"))?;

    let fichier = std::fs::File::open(&temporaire)
        .map_err(|e| format!("Ouverture du fichier temporaire : {e}"))?;
    fichier.sync_all().map_err(|e| format!("Synchronisation impossible : {e}"))?;
    drop(fichier);

    std::fs::rename(&temporaire, destination)
        .map_err(|e| format!("Remplacement du fichier impossible : {e}"))?;
    Ok(())
}

fn restaurer_fichier(destination: &Path, backup: &Path) -> Result<(), String> {
    remplacer_fichier(destination, backup)
}
```

En tête du fichier, ajouter les imports :

```rust
use crate::core::db::migrations;
use std::path::PathBuf;
```

Retirer la fonction `importer` (plus utilisée) et `nom_fichier_sauvegarde` reste inchangé. Vérifier `commandes.rs` : `io_cmd::importer` n'existe plus — voir étape 6.

- [ ] **Step 5: `message.rs` — variant de résultat**

Dans `src/app/message.rs`, section Import/Export, ajouter :

```rust
    ConfirmImport(String),
    /// Résultat de l'import en arrière-plan (AB-003, AB-011).
    ImportResult(Result<(), String>),
    CancelImport,
```

- [ ] **Step 6: `update.rs` — import en tâche de fond**

Remplacer `Message::ConfirmImport` (lignes 594-615) :

```rust
        Message::ConfirmImport(path_str) => {
            let source = std::path::PathBuf::from(&path_str);
            let Some(chemin_actuel) = state.db.as_ref().map(|p| p.path.clone()) else {
                state.import_error = Some("Base de données non initialisée.".into());
                return Task::none();
            };

            Task::perform(
                async move { io_cmd::importer_en_arriere_plan(chemin_actuel, source) },
                Message::ImportResult,
            )
        }
        Message::ImportResult(resultat) => {
            state.show_import_confirm = false;
            state.import_file_path = None;

            // Réouverture systématique : la connexion vivante peut pointer
            // vers un inode remplacé par l'import ou la restauration.
            let reouverture = state
                .db
                .as_ref()
                .map(|p| p.path.clone())
                .ok_or_else(|| "Base de données non initialisée.".to_string())
                .and_then(|chemin| DatabasePool::open(&chemin));

            match (resultat, reouverture) {
                (Ok(()), Ok(pool)) => {
                    state.db = Some(pool);
                    match state.load_data() {
                        Ok(()) => state.notification = Some(Notification::succes("Import réussi.")),
                        Err(e) => state.notification = Some(Notification::erreur(format!(
                            "Import effectué, mais rechargement impossible : {e}"
                        ))),
                    }
                }
                (Ok(()), Err(e)) => {
                    state.notification = Some(Notification::erreur(format!(
                        "Import effectué, mais réouverture impossible : {e}"
                    )));
                }
                (Err(e), Ok(pool)) => {
                    state.db = Some(pool);
                    state.import_error = Some(format!("Erreur d'import : {e}"));
                    state.notification =
                        Some(Notification::erreur(format!("L'import a échoué : {e}")));
                }
                (Err(e), Err(e2)) => {
                    state.import_error = Some(format!("Erreur d'import : {e}"));
                    state.notification = Some(Notification::erreur(format!(
                        "L'import a échoué : {e} ; réouverture impossible : {e2}"
                    )));
                }
            }
            Task::none()
        }
```

Ajouter l'import `DatabasePool` en tête de `update.rs` (déjà présent ligne 3) et `std::path::PathBuf` (déjà utilisé). Supprimer la fonction `perform_import` (lignes 759-787) et son import de `DatabasePool::validate_sqlite_file` — supprimer aussi `use crate::core::db::pool::DatabasePool;` seulement si plus utilisé ailleurs (il reste utilisé par `Message::ImportResult` → le garder).

Dans `src/modules/import_export/commandes.rs`, remplacer `importer` :

```rust
pub fn importer(pool: &mut DatabasePool, source: &Path) -> Result<(), String> {
    service::importer(pool, source)
}
```

par :

```rust
pub fn importer_en_arriere_plan(
    chemin_actuel: std::path::PathBuf,
    source: std::path::PathBuf,
) -> Result<(), String> {
    service::importer_en_arriere_plan(chemin_actuel, source)
}
```

- [ ] **Step 7: Vérifier**

Run: `cargo test --lib modules::import_export` — Expected: PASS (5 tests).
Run: `cargo test --lib` — Expected: PASS.
Run: `cargo clippy --all-targets --all-features -- -D warnings` — Expected: code 0.

- [ ] **Step 8: Commit**

```bash
git add src/core/db/pool.rs src/modules/import_export/ src/app/message.rs src/app/update.rs
git commit -m "fix: import validé en lecture seule et remplacement atomique en arrière-plan (AB-002, AB-003, AB-011)"
```

---

### Task 9: AB-004 — Réinitialisation transactionnelle

**Files:**
- Modify: `src/modules/parametres/repository.rs` (`reset_all_data`)
- Test: `src/modules/parametres/tests/mod.rs`

- [ ] **Step 1: Test rouge — reset complet et rollback**

Dans `src/modules/parametres/tests/mod.rs`, ajouter :

```rust
use crate::domaine::argent::Money;
use crate::domaine::parametres::AppSettings;
use crate::modules::parametres::repository as repo;
use crate::modules::recurrences::repository as regles_repo;
use crate::modules::transactions::repository as tx_repo;

fn inserer_scenario_avec_regle(pool: &DatabasePool) {
    // Catégorie personnalisée référencée par une règle ET une transaction :
    // le scénario exact de l'audit (AB-004).
    let maintenant = chrono::Utc::now().to_rfc3339();
    pool.conn
        .execute(
            "INSERT INTO categories (id, kind, name, icon, color, sort_order, is_default, is_active, created_at, updated_at)
             VALUES ('perso', 'expense', 'Perso', 'x', '#3B82F6', 99, 0, 1, ?1, ?1)",
            rusqlite::params![&maintenant],
        )
        .unwrap();

    let regle = crate::domaine::recurrence::RecurringRule {
        id: "regle-1".into(),
        kind: crate::domaine::transaction::TransactionKind::Expense,
        label: "Abonnement".into(),
        amount: Money::from_cents(99900),
        category_id: "perso".into(),
        day_of_month: 5,
        start: (2026, 8),
        end: None,
        note: None,
        is_active: true,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    regles_repo::creer(pool, &regle).unwrap();

    let tx = crate::tests_commun::donnees_test::transaction_test(
        "Perso achat",
        5000,
        crate::domaine::transaction::TransactionKind::Expense,
        crate::domaine::transaction::TransactionStatus::Pending,
        "2026-08-01",
        "perso",
    );
    tx_repo::insert(pool, &tx).unwrap();

    let parametres = AppSettings {
        current_balance: Money::from_cents(10000),
        ..Default::default()
    };
    repo::insert_settings(pool, &parametres).unwrap();
}

fn compter(pool: &DatabasePool, table: &str) -> i64 {
    pool.conn
        .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))
        .unwrap()
}

/// Le reset efface tout : transactions, paramètres, règles et catégories
/// personnalisées, dans une seule transaction (AB-004).
#[test]
fn le_reset_efface_tout_y_compris_les_regles() {
    let pool = base_temporaire::creer_base_test();
    inserer_scenario_avec_regle(&pool);

    repo::reset_all_data(&pool).unwrap();

    assert_eq!(compter(&pool, "transactions"), 0);
    assert_eq!(compter(&pool, "app_settings"), 0);
    assert_eq!(compter(&pool, "recurring_rules"), 0);
    // 35 catégories par défaut (26 dépenses + 9 revenus) ; aucune personnalisée.
    assert_eq!(compter(&pool, "categories"), 35);
}

/// Sur une base non inscriptible, le reset échoue sans rien effacer (AB-004).
#[test]
fn le_reset_echoue_sans_effet_sur_base_non_inscriptible() {
    let pool = base_temporaire::creer_base_test();
    inserer_scenario_avec_regle(&pool);

    pool.conn.execute_batch("PRAGMA query_only = ON;").unwrap();
    assert!(repo::reset_all_data(&pool).is_err());
    pool.conn.execute_batch("PRAGMA query_only = OFF;").unwrap();

    assert_eq!(compter(&pool, "transactions"), 1);
    assert_eq!(compter(&pool, "recurring_rules"), 1);
    assert_eq!(compter(&pool, "app_settings"), 1);
}
```

Note : la catégorie `perso` est `is_default = 0` → le compteur attendu après reset est 35 (26 dépenses + 9 revenus). Vérifier la valeur réelle des catégories par défaut avec `cargo test --lib` si besoin.

- [ ] **Step 2: Vérifier l'échec**

Run: `cargo test --lib modules::parametres::tests::le_reset_efface_tout_y_compris_les_regles`
Expected: FAIL — `recurring_rules` non vidée (count 1).

- [ ] **Step 3: Implémenter**

Dans `src/modules/parametres/repository.rs`, remplacer `reset_all_data` (lignes 81-90) :

```rust
/// Efface toutes les données d'une transaction unique : règles récurrentes,
/// transactions, catégories personnalisées, paramètres. En cas d'erreur, tout
/// est annulé — jamais de suppression partielle (AB-004).
pub fn reset_all_data(pool: &DatabasePool) -> Result<(), String> {
    let tx = pool
        .conn
        .unchecked_transaction()
        .map_err(|e| format!("Démarrage de la transaction de réinitialisation : {e}"))?;

    tx.execute("DELETE FROM recurring_rules", [])
        .map_err(|e| format!("Erreur de réinitialisation des règles : {e}"))?;
    tx.execute("DELETE FROM transactions", [])
        .map_err(|e| format!("Erreur de réinitialisation des transactions : {e}"))?;
    tx.execute("DELETE FROM categories WHERE is_default = 0", [])
        .map_err(|e| format!("Erreur de réinitialisation des catégories : {e}"))?;
    tx.execute("DELETE FROM app_settings", [])
        .map_err(|e| format!("Erreur de réinitialisation des paramètres : {e}"))?;

    tx.commit()
        .map_err(|e| format!("Erreur de commit de la réinitialisation : {e}"))?;
    Ok(())
}
```

- [ ] **Step 4: Vérifier**

Run: `cargo test --lib modules::parametres` — Expected: PASS.
Run: `cargo test --lib` — Expected: PASS.
Run: `cargo clippy --all-targets --all-features -- -D warnings` — Expected: code 0.

- [ ] **Step 5: Commit**

```bash
git add src/modules/parametres/
git commit -m "fix: réinitialisation transactionnelle incluant les règles (AB-004)"
```

---

### Task 10: AB-010 — Permissions locales

**Files:**
- Modify: `src/core/config.rs`
- Modify: `src/core/db/pool.rs` (chmod + log)
- Test: `src/core/config.rs` (`#[cfg(test)]`), `src/core/db/pool.rs` (`#[cfg(test)]`)

**Interfaces:**
- Produces: `config::restreindre_permissions(chemin: &Path) -> Result<(), String>` (no-op hors unix) ; `DatabasePool::open` restreint le fichier à 0600 (unix) et ne loggue plus le chemin en info.

- [ ] **Step 1: Tests rouges (unix)**

Dans `src/core/config.rs`, ajouter un module de tests :

```rust
#[cfg(test)]
mod tests {
    use super::*;

    /// Le répertoire de données et les fichiers qu'il contient ne doivent pas
    /// être lisibles par les autres utilisateurs locaux (AB-010).
    #[test]
    #[cfg(unix)]
    fn le_repertoire_est_prive() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        restreindre_permissions(dir.path()).unwrap();
        let mode = std::fs::metadata(dir.path()).unwrap().permissions().mode();
        assert_eq!(mode & 0o077, 0, "mode observé : {mode:o}");
    }
}
```

Dans `src/core/db/pool.rs`, ajouter un module de tests :

```rust
#[cfg(test)]
mod tests {
    use super::*;

    /// La base créée ne doit pas être lisible par les autres utilisateurs
    /// locaux (AB-010).
    #[test]
    #[cfg(unix)]
    fn la_base_est_privee() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let chemin = dir.path().join("base.sqlite");
        let pool = DatabasePool::open(&chemin).unwrap();
        drop(pool);
        let mode = std::fs::metadata(&chemin).unwrap().permissions().mode();
        assert_eq!(mode & 0o077, 0, "mode observé : {mode:o}");
    }
}
```

- [ ] **Step 2: Vérifier les échecs**

Run: `cargo test --lib core::config::tests core::db::pool::tests`
Expected: FAIL — `restreindre_permissions` inconnue ; mode 0644 pour la base.

- [ ] **Step 3: Implémenter — config**

Remplacer `src/core/config.rs` :

```rust
use std::path::{Path, PathBuf};

pub const APP_NAME: &str = "afterbudget";

/// Restreint l'accès à un répertoire de données : 0700 sur Unix, sans effet
/// sur Windows (pas de mode POSIX) (AB-010).
pub fn restreindre_permissions(chemin: &Path) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(chemin, std::fs::Permissions::from_mode(0o700))
            .map_err(|e| format!("Restriction des permissions impossible : {e}"))?;
    }
    #[cfg(not(unix))]
    let _ = chemin;
    Ok(())
}

/// Restreint l'accès à un fichier de données : 0600 sur Unix (AB-010).
pub fn restreindre_permissions_fichier(chemin: &Path) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(chemin, std::fs::Permissions::from_mode(0o600))
            .map_err(|e| format!("Restriction des permissions impossible : {e}"))?;
    }
    #[cfg(not(unix))]
    let _ = chemin;
    Ok(())
}

pub fn app_data_dir() -> Result<PathBuf, String> {
    let dir = dirs::data_dir().ok_or("Répertoire de données introuvable.")?;
    let app_dir = dir.join(APP_NAME);
    std::fs::create_dir_all(&app_dir)
        .map_err(|e| format!("Création du répertoire impossible : {e}"))?;
    restreindre_permissions(&app_dir)?;
    Ok(app_dir)
}

pub fn database_path() -> Result<PathBuf, String> {
    Ok(app_data_dir()?.join("afterbudget.sqlite"))
}

pub fn backup_dir() -> Result<PathBuf, String> {
    Ok(app_data_dir()?.join("backups"))
}
```

(la signature `Result` anticipe AB-013, Task 11).

- [ ] **Step 4: Implémenter — pool (chmod + log)**

Dans `src/core/db/pool.rs`, `open` (lignes 11-29) :

```rust
    pub fn open(path: &Path) -> Result<Self, String> {
        let conn = rusqlite::Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_CREATE
                | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(|e| format!("Impossible d'ouvrir la base : {}", e))?;

        conn.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL;")
            .map_err(|e| format!("Erreur configuration SQLite : {}", e))?;

        crate::core::config::restreindre_permissions_fichier(path)?;

        tracing::debug!("Base de données ouverte.");

        Ok(Self {
            conn,
            path: path.to_path_buf(),
        })
    }
```

- [ ] **Step 5: Adapter les appelants existants de `app_data_dir`/`database_path`**

Run: `cargo check --lib` — Expected: erreurs sur `src/main.rs:22` et `src/app/state.rs:193`. Dans `src/app/state.rs`, `AppState::new` (ligne 191-194) :

```rust
    pub fn new() -> Self {
        let now = chrono::Utc::now();
        let db_path = crate::core::config::database_path()
            .unwrap_or_else(|_| std::path::PathBuf::from("afterbudget.sqlite"));
```

`src/main.rs` sera traité dans Task 11 (AB-013).

- [ ] **Step 6: Vérifier**

Run: `cargo test --lib` — Expected: PASS.
Run: `cargo clippy --all-targets --all-features -- -D warnings` — Expected: code 0.

- [ ] **Step 7: Commit**

```bash
git add src/core/config.rs src/core/db/pool.rs src/app/state.rs
git commit -m "fix: permissions privées sur le répertoire et la base (AB-010)"
```

---

### Task 11: AB-013 — Démarrage sans panic

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Réécrire `main.rs`**

Remplacer les lignes 22-39 :

```rust
    let db_path = match config::database_path() {
        Ok(chemin) => chemin,
        Err(e) => {
            tracing::error!("{}", e);
            eprintln!("AfterBudget : {}", e);
            std::process::exit(1);
        }
    };
    let db = match DatabasePool::open(&db_path) {
        Ok(db) => db,
        Err(e) => {
            tracing::error!("{}", e);
            eprintln!("AfterBudget : {}", e);
            std::process::exit(1);
        }
    };

    if let Err(e) = migrations::run_migrations(&db.conn) {
        tracing::error!("{}", e);
        eprintln!("AfterBudget : {}", e);
        std::process::exit(1);
    }

    let mut state = AppState::new();
    state.db_path = db.path.to_string_lossy().to_string();
    state.db = Some(db);

    if let Err(e) = state.load_data() {
        tracing::error!("Chargement initial impossible : {}", e);
        eprintln!("AfterBudget : le chargement initial a échoué ({}) — arrêt.", e);
        std::process::exit(1);
    }
```

- [ ] **Step 2: Vérifier**

Run: `cargo check --all-targets` — Expected: OK.
Run: `cargo test --lib` — Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git add src/main.rs
git commit -m "fix: erreurs de démarrage rendues sans panic (AB-013)"
```

---

### Task 12: AB-011 — Statistiques en SQL et benchmark volumineux

**Files:**
- Modify: `src/modules/transactions/repository.rs` (`count_by_kind`)
- Modify: `src/modules/statistiques/service.rs`
- Modify: `src/modules/transactions/tests/mod.rs` (benchmark `#[ignore]`)

**Interfaces:**
- Produces: `transactions::repository::count_by_kind(pool, year, month, kind) -> Result<i64, String>`.

- [ ] **Step 1: Test de non-régression — résultats identiques**

Dans `src/modules/transactions/tests/mod.rs`, ajouter :

```rust
    /// Le comptage SQL doit recouper le chargement complet : les statistiques
    /// ne doivent plus charger toutes les lignes (AB-011).
    #[test]
    fn le_comptage_sql_recoupe_le_chargement() {
        let pool = base_temporaire::creer_base_test();
        for i in 0..50 {
            let kind = if i % 2 == 0 {
                TransactionKind::Income
            } else {
                TransactionKind::Expense
            };
            let tx = donnees_test::transaction_test(
                &format!("Opération {i}"),
                1000,
                kind,
                TransactionStatus::Pending,
                "2026-08-01",
                if kind == TransactionKind::Income { "salaire" } else { "autre" },
            );
            repo::insert(&pool, &tx).unwrap();
        }
        let lignes = repo::find_all_by_month(&pool, 2026, 8).unwrap();
        let par_kind = |k: &str| repo::count_by_kind(&pool, 2026, 8, k).unwrap();
        assert_eq!(par_kind("income") as usize, lignes.iter().filter(|t| t.kind == TransactionKind::Income).count());
        assert_eq!(par_kind("expense") as usize, lignes.iter().filter(|t| t.kind == TransactionKind::Expense).count());
    }
```

- [ ] **Step 2: Vérifier l'échec**

Run: `cargo test --lib modules::transactions::tests::le_comptage_sql_recoupe_le_chargement`
Expected: FAIL — `count_by_kind` inconnue.

- [ ] **Step 3: Implémenter `count_by_kind`**

Dans `src/modules/transactions/repository.rs`, après `sum_by_kind` (ligne 338) :

```rust
pub fn count_by_kind(pool: &DatabasePool, year: i32, month: u32, kind: &str) -> Result<i64, String> {
    let start_date = format!("{:04}-{:02}-01", year, month);
    let end_day = last_day_of_month(year, month);
    let end_date = format!("{:04}-{:02}-{:02}", year, month, end_day);

    let result: Result<i64, _> = pool.conn.query_row(
        "SELECT COUNT(*) FROM transactions
         WHERE transaction_date >= ?1 AND transaction_date <= ?2
         AND kind = ?3",
        params![start_date, end_date, kind],
        |row| row.get(0),
    );

    result.map_err(|e| format!("Erreur de comptage : {}", e))
}
```

- [ ] **Step 4: Statistiques sans chargement complet**

Dans `src/modules/statistiques/service.rs`, remplacer le bloc `let txs = transactions::find_by_month(...)` (lignes 27-36) :

```rust
    let income_count = transactions::count_by_kind(pool, year, month, "income")?;
    let expense_count = transactions::count_by_kind(pool, year, month, "expense")?;
```

et dans la construction de `MonthlyStatistics` (lignes 43-56), remplacer :

```rust
    Ok(MonthlyStatistics {
        total_income,
        total_expenses,
        balance,
        completed_income,
        pending_income,
        completed_expenses,
        pending_expenses,
        transaction_count: (income_count + expense_count) as usize,
        income_count: income_count as usize,
        expense_count: expense_count as usize,
        expenses_by_category,
        income_by_category,
    })
```

- [ ] **Step 5: Benchmark de non-régression**

Dans `src/modules/transactions/tests/mod.rs`, ajouter :

```rust
    /// Benchmark volumineux : 100 000 transactions dans le mois. Hors CI par
    /// défaut (`cargo test -- --ignored`) ; seuil large pour rester stable.
    #[test]
    #[ignore = "benchmark volumineux : lancer avec cargo test -- --ignored"]
    fn perf_grosses_bases() {
        let pool = base_temporaire::creer_base_test();
        let maintenant = "2026-08-01T00:00:00Z";

        {
            let tx = pool.conn.unchecked_transaction().expect("transaction");
            {
                let mut stmt = tx
                    .prepare(
                        "INSERT INTO transactions (id, kind, label, amount_cents, transaction_date, status,
                            category_id, note, recurring_rule_id, created_at, updated_at)
                         VALUES (?1, 'expense', ?2, ?3, '2026-08-15', 'pending',
                            'autre', NULL, NULL, ?4, ?4)",
                    )
                    .expect("préparation");
                for i in 0..100_000i64 {
                    stmt.execute(rusqlite::params![
                        uuid::Uuid::new_v4().to_string(),
                        format!("Dépense {i}"),
                        100 + (i % 5000),
                        maintenant,
                    ])
                    .expect("insertion");
                }
            }
            tx.commit().expect("commit");
        }

        let debut_lecture = std::time::Instant::now();
        let lignes = repo::find_all_by_month(&pool, 2026, 8).expect("lecture");
        let duree_lecture = debut_lecture.elapsed();
        assert_eq!(lignes.len(), 100_000);
        assert!(
            duree_lecture.as_secs() < 2,
            "lecture trop lente : {duree_lecture:?}"
        );

        let debut_stats = std::time::Instant::now();
        let stats = crate::modules::statistiques::service::calculer_statistiques_mensuelles(
            &pool, 2026, 8,
        )
        .expect("statistiques");
        let duree_stats = debut_stats.elapsed();
        assert_eq!(stats.transaction_count, 100_000);
        assert!(
            duree_stats.as_secs() < 2,
            "statistiques trop lentes : {duree_stats:?}"
        );
    }
```

- [ ] **Step 6: Vérifier**

Run: `cargo test --lib` — Expected: PASS.
Run: `cargo test --lib -- --ignored perf_grosses_bases` — Expected: PASS (< 2 s par phase ; si la machine est lente, passer le seuil à 5 s).
Run: `cargo clippy --all-targets --all-features -- -D warnings` — Expected: code 0.

- [ ] **Step 7: Commit**

```bash
git add src/modules/transactions/repository.rs src/modules/statistiques/
git commit -m "perf: statistiques par comptage SQL et benchmark 100k (AB-011)"
```

---

### Task 13: AB-012 — Architecture : update.rs sans accès direct aux repositories

**Files:**
- Modify: `src/modules/transactions/commandes.rs` (`trouver`)
- Modify: `src/modules/transactions/service.rs` (`trouver_transaction`)
- Modify: `src/app/update.rs` (`OpenEditTransaction`, `DeleteTransaction` via `tx_cmd::trouver`)
- Delete: `src/modules/categories/dtos/` (creer_categorie.rs, modifier_categorie.rs, mod.rs)
- Delete: `src/modules/categories/validateurs.rs`, `src/modules/categories/mappers.rs`
- Delete: `src/modules/import_export/dtos/`, `src/modules/import_export/mappers.rs`
- Modify: `src/modules/categories/mod.rs`, `src/modules/import_export/mod.rs`
- Modify: `src/modules/categories/commandes.rs`, `src/modules/categories/service.rs` (retirer `par_type`/`categories_par_type` si inutilisés)

**Interfaces:**
- Produces: `tx_cmd::trouver(pool, id) -> Result<Option<Transaction>, String>`.

- [ ] **Step 1: Vérifier les usages morts**

Run: `grep -rn "par_type\|categories_par_type\|CreerCategorieDto\|ModifierCategorieDto\|ImporterSauvegardeDto" src/`
Expected: seules les définitions du module catégories (commandes.rs/service.rs pour par_type ; validateurs.rs pour les DTO). Si des usages apparaissent ailleurs, les conserver et adapter.

- [ ] **Step 2: Ajouter `tx_cmd::trouver`**

Dans `src/modules/transactions/service.rs`, ajouter :

```rust
pub fn trouver_transaction(pool: &DatabasePool, id: &str) -> Result<Option<Transaction>, String> {
    repo::find_by_id(pool, id)
}
```

Dans `src/modules/transactions/commandes.rs`, ajouter :

```rust
pub fn trouver(pool: &DatabasePool, id: &str) -> Result<Option<Transaction>, String> {
    service::trouver_transaction(pool, id)
}
```

- [ ] **Step 3: Brancher dans update.rs**

Remplacer dans `Message::OpenEditTransaction` (ligne 217) :

```rust
                if let Ok(Some(tx)) = transaction_repo::find_by_id(db, &id) {
```

par :

```rust
                if let Ok(Some(tx)) = tx_cmd::trouver(db, &id) {
```

Remplacer dans `Message::DeleteTransaction` (ligne 471) :

```rust
                if let Ok(Some(tx)) = transaction_repo::find_by_id(db, &id) {
```

par :

```rust
                if let Ok(Some(tx)) = tx_cmd::trouver(db, &id) {
```

Supprimer l'import `use crate::modules::transactions::repository as transaction_repo;` (ligne 15) si plus référencé.

- [ ] **Step 4: Supprimer les fichiers morts**

```bash
git rm src/modules/categories/dtos/creer_categorie.rs src/modules/categories/dtos/modifier_categorie.rs src/modules/categories/dtos/mod.rs src/modules/categories/validateurs.rs src/modules/categories/mappers.rs src/modules/import_export/dtos/importer_sauvegarde.rs src/modules/import_export/dtos/mod.rs src/modules/import_export/mappers.rs
```

Adapter `src/modules/categories/mod.rs` :

```rust
pub mod commandes;
pub mod composants;
pub mod repository;
pub mod service;

#[cfg(test)]
mod tests;
```

Adapter `src/modules/import_export/mod.rs` :

```rust
pub mod commandes;
pub mod service;
pub mod views;

#[cfg(test)]
mod tests;
```

- [ ] **Step 5: Retirer `par_type` si mort**

Dans `src/modules/categories/commandes.rs`, retirer `par_type` ; dans `service.rs`, retirer `categories_par_type` si aucun usage (vérifié à l'étape 1). Vérifier `find_by_kind` de `repository.rs` : conserver (API publique du repository, utilisée par les vues ?) — vérifier avec `grep -rn "find_by_kind" src/` et supprimer seulement s'il n'y a aucun appelant.

- [ ] **Step 6: Vérifier**

Run: `cargo test --all-targets` — Expected: PASS.
Run: `cargo clippy --all-targets --all-features -- -D warnings` — Expected: code 0.
Run: `grep -n "repository as\|::repository" src/app/update.rs` — Expected: aucune occurrence.

- [ ] **Step 7: Commit**

```bash
git add -A src/
git commit -m "refactor: update.rs passe par les commandes, code mort supprimé (AB-012)"
```

---

### Task 14: P2 — `last_export_date` réellement persisté

**Files:**
- Modify: `src/core/db/modeles.rs`
- Modify: `src/modules/parametres/repository.rs`
- Modify: `src/modules/parametres/service.rs` (`marquer_dernier_export`)
- Modify: `src/app/update.rs` (ExportDatabase → service)
- Test: `src/modules/parametres/tests/mod.rs`

**Interfaces:**
- Consumes: colonne `last_export_date` (migration v3, Task 7).
- Produces: `parametres::service::marquer_dernier_export(pool) -> Result<(), String>`.

- [ ] **Step 1: Test rouge — persistance après réouverture**

Dans `src/modules/parametres/tests/mod.rs`, ajouter :

```rust
/// La date du dernier export doit survivre au redémarrage (AB-012/P2).
#[test]
fn la_date_du_dernier_export_survit_a_la_reouverture() {
    let repertoire = tempfile::tempdir().unwrap();
    let chemin = repertoire.path().join("base.sqlite");

    let pool = DatabasePool::open(&chemin).unwrap();
    crate::core::db::migrations::run_migrations(&pool.conn).unwrap();
    let parametres = AppSettings {
        current_balance: Money::from_cents(10000),
        ..Default::default()
    };
    repo::insert_settings(&pool, &parametres).unwrap();
    service::marquer_dernier_export(&pool).unwrap();
    drop(pool);

    let relu = DatabasePool::open(&chemin).unwrap();
    let parametres = repo::get_settings(&relu).unwrap().unwrap();
    assert!(
        parametres.last_export_date.is_some(),
        "la date doit être persistée"
    );
}
```

- [ ] **Step 2: Vérifier l'échec**

Run: `cargo test --lib modules::parametres::tests::la_date_du_dernier_export_survit_a_la_reouverture`
Expected: FAIL — colonne inconnue (`SELECT ... last_export_date`) ou `marquer_dernier_export` inconnue.

- [ ] **Step 3: `modeles.rs`**

Ajouter dans `SettingsRow` :

```rust
    pub last_export_date: Option<String>,
```

- [ ] **Step 4: `parametres/repository.rs`**

`get_settings` : ajouter la colonne au `SELECT` et à la construction :

```rust
        "SELECT id, current_balance_cents, overdraft_limit_cents, currency_code, locale, theme,
                balance_updated_at, onboarding_completed, created_at, updated_at, last_export_date
         FROM app_settings WHERE id = 1",
```

et dans la closure :

```rust
                onboarding_completed: row.get::<_, i32>(7)? != 0,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
                last_export_date: row.get(10)?,
```

`insert_settings` :

```rust
            "INSERT INTO app_settings (id, current_balance_cents, overdraft_limit_cents, currency_code, locale, theme,
             balance_updated_at, onboarding_completed, last_export_date, created_at, updated_at)
             VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)",
            params![
                s.current_balance.cents,
                s.overdraft_limit.cents,
                s.currency_code,
                s.locale,
                s.theme,
                s.balance_updated_at,
                s.onboarding_completed as i32,
                s.last_export_date,
                &now,
            ],
```

`update_settings` :

```rust
            "UPDATE app_settings SET current_balance_cents = ?1, overdraft_limit_cents = ?2,
             currency_code = ?3, locale = ?4, theme = ?5, balance_updated_at = ?6,
             onboarding_completed = ?7, last_export_date = ?8, updated_at = ?9
             WHERE id = 1",
            params![
                s.current_balance.cents,
                s.overdraft_limit.cents,
                s.currency_code,
                s.locale,
                s.theme,
                s.balance_updated_at,
                s.onboarding_completed as i32,
                s.last_export_date,
                &now,
            ],
```

`row_to_settings` :

```rust
        last_export_date: row.last_export_date.clone(),
```

- [ ] **Step 5: `parametres/service.rs`**

Ajouter :

```rust
pub fn marquer_dernier_export(pool: &DatabasePool) -> Result<(), String> {
    let mut settings = repo::get_settings(pool)?.unwrap_or_default();
    settings.last_export_date = Some(chrono::Utc::now().to_rfc3339());
    repo::update_settings(pool, &settings)
}
```

- [ ] **Step 6: `update.rs` ExportDatabase**

Remplacer le bloc (déjà modifié en Task 4) :

```rust
                    Ok(()) => {
                        if let Some(mut s) = params_repo::get_settings(db)
                            .map_err(|e| format!("Relecture des paramètres impossible : {e}"))?
                        {
                            s.last_export_date = Some(chrono::Utc::now().to_rfc3339());
                            params_repo::update_settings(db, &s)?;
                            state.settings = Some(s);
                        }
```

par :

```rust
                    Ok(()) => {
                        parametres_service::marquer_dernier_export(db)?;
                        state.settings = parametres_service::obtenir_parametres(db).ok().flatten();
```

Puis supprimer l'import `use crate::modules::parametres::repository as params_repo;` (ligne 9) si plus référencé.

- [ ] **Step 7: Vérifier**

Run: `cargo test --lib` — Expected: PASS.
Run: `cargo clippy --all-targets --all-features -- -D warnings` — Expected: code 0.

- [ ] **Step 8: Commit**

```bash
git add src/core/db/modeles.rs src/modules/parametres/ src/app/update.rs
git commit -m "feat: persister la date du dernier export (P2)"
```

---

### Task 15: P2 — Dépendances (validator), audit.toml, profils Cargo

**Files:**
- Modify: `Cargo.toml`
- Create: `.cargo/audit.toml`
- Modify: `Cargo.lock` (via cargo update)

- [ ] **Step 1: Mettre à jour `validator`**

Dans `Cargo.toml` :

```toml
validator = { version = "0.19", features = ["derive"] }
```

Run: `cargo update -p validator` — Expected: `idna 1.x` dans `Cargo.lock`.
Run: `cargo build` — Expected: OK (les `#[derive(Validate)]` existants compilent avec 0.19 ; si une incompatibilité d'attribut apparaît, l'ajuster selon le message d'erreur).

- [ ] **Step 2: Exception `lru`**

Créer `.cargo/audit.toml` :

```toml
[advisories]
# RUSTSEC-2026-0002 : lru 0.12.5 (IterMut unsound) est piné par iced_glyphon
# 0.6.0, dépendance d'iced 0.13 (dernière version publiée). Aucun correctif
# upstream disponible ; le graphe AfterBudget n'utilise ni iter_mut ni la
# fonctionnalité concernée. À réévaluer à chaque montée d'iced.
ignore = ["RUSTSEC-2026-0002"]
```

- [ ] **Step 3: Profils et metadata**

Dans `Cargo.toml`, ajouter :

```toml
rust-version = "1.85"

[profile.release]
lto = "thin"
codegen-units = 1
strip = "symbols"

[package.metadata]
repository = "https://github.com/alexandrebouttierdev/afterbudget"
keywords = ["budget", "finance", "desktop", "sqlite"]
categories = ["command-line-utilities"]
```

- [ ] **Step 4: Vérifier**

Run: `cargo build --release` — Expected: OK.
Run: `cargo audit --file Cargo.lock` — Expected: code 0 (si `cargo-audit` est installé ; sinon noter la commande pour CI).

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml Cargo.lock .cargo/audit.toml
git commit -m "build: validator 0.19 (idna 1.x), exception lru documentée, profils release (AB-014)"
```

---

### Task 16: P2 — CI GitHub Actions et documentation

**Files:**
- Create: `.github/workflows/ci.yml`
- Modify: `README.md`
- Modify: `docs/ARCHITECTURE.md`
- Modify: `docs/DATABASE.md`

- [ ] **Step 1: Workflow CI**

Créer `.github/workflows/ci.yml` :

```yaml
name: CI

on:
  push:
    branches: [master]
  pull_request:

env:
  CARGO_TERM_COLOR: always

jobs:
  linux:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      - uses: Swatinem/rust-cache@v2
      - name: Format
        run: cargo fmt --all -- --check
      - name: Clippy
        run: cargo clippy --locked --all-targets --all-features -- -D warnings
      - name: Tests
        run: cargo test --locked --all-targets --all-features
      - name: Doc-tests
        run: cargo test --locked --doc --all-features
      - name: Release
        run: cargo build --locked --release --all-features
      - name: Audit
        uses: rustsec/audit-check@v2
        with:
          token: ${{ secrets.GITHUB_TOKEN }}

  macos-build:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Build release
        run: cargo build --locked --release --all-features

  windows-build:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Build release
        run: cargo build --locked --release --all-features
```

Note : macOS/Windows ne font que **compiler** (tests Linux sur le job `linux`) — documenté dans le README.

- [ ] **Step 2: README**

Modifier `README.md` :
- Table « Technologies » : `validator 0.19+`.
- Ajouter une section « Dépendances et sécurité » après « Tests » :

```markdown
## Dépendances et sécurité

- `cargo audit` est vert (`.cargo/audit.toml`).
- Exception documentée : `lru 0.12.5` (RUSTSEC-2026-0002) est épinglée par
  `iced_glyphon 0.6.0` (Iced 0.13, dernière version) ; la fonctionnalité
  concernée (`IterMut`) n'est pas utilisée par AfterBudget.
```

- Compléter « Confidentialité » :

```markdown
Les fichiers locaux sont protégés : répertoire de données en `0700` et base
SQLite en `0600` sur Linux/macOS (pas de mode POSIX sous Windows). La base
n'est pas chiffrée : tout utilisateur local ayant accès au système peut la
lire s'il a les droits sur le répertoire.
```

- Mettre à jour « Compatibilité » et les prérequis (`rust-version = "1.85"`).

- [ ] **Step 3: ARCHITECTURE.md**

Dans `docs/ARCHITECTURE.md` :
- Retirer de l'arbre `categories/` : `mappers.rs`, `dtos/` (creer/modifier), `validateurs.rs` ; retirer `import_export/` : `mappers.rs`, `dtos/`.
- Remplacer la phrase « Chaque module métier contient ses DTOs, validateurs, … » du README par une liste exacte des modules et de leurs fichiers réels.
- Ajouter au schéma des couches une note : « `app/update.rs` orchestre l'état ; toute persistance passe par les commandes/services des modules — aucun accès repository ni SQL dans `update.rs` ».

- [ ] **Step 4: DATABASE.md**

Dans `docs/DATABASE.md` :
- Table `transactions` : ajouter la colonne `recurring_rule_id | TEXT NULL FK | Règle productrice`.
- Nouvelle table `recurring_rules` (copier le schéma de `migrations/0002_recurrences.sql`).
- Table `app_settings` : ajouter `last_export_date | TEXT NULL | Dernier export réussi`.
- Section Migrations : lister `0001_initial.sql`, `0002_recurrences.sql`, `0003_last_export_date.sql` ; préciser : « chaque migration est appliquée dans une transaction unique avec son enregistrement de version ; les ALTER sont rejoués uniquement si la colonne manque (idempotence) ».

- [ ] **Step 5: Vérifier**

Run: `cargo fmt --all -- --check` — Expected: code 0.
Run: `git diff --check` — Expected: code 0.

- [ ] **Step 6: Commit**

```bash
git add .github/ README.md docs/
git commit -m "docs: CI Linux + compilations macOS/Windows, documentation alignée (P2)"
```

---

### Task 17: Portes finales

- [ ] **Step 1: Portes complètes**

```bash
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-targets --all-features
cargo test --doc --locked --all-features
cargo build --locked --release --all-features
cargo audit --file Cargo.lock
```

Expected : toutes code 0 ; tests 224+ passés, 0 échec.

- [ ] **Step 2: Vérifications d'invariants**

```bash
grep -rn "from_euros\|to_euros_f64" src/    # aucune
grep -n "let _ =\|\.ok()" src/app/update.rs # aucun sur un chemin d'écriture
grep -n "repository as\|::repository" src/app/update.rs  # aucune
grep -rn "CreerCategorieDto\|ImporterSauvegardeDto" src/ # aucune
```

- [ ] **Step 3: Vérification manuelle (si affichage disponible)**

Run: `cargo run` — Expected: fenêtre s'ouvre, onboarding s'affiche.

- [ ] **Step 4: Commit final (si des retouches ont été nécessaires)**

```bash
git add -A
git commit -m "chore: portes finales vertes"
```
