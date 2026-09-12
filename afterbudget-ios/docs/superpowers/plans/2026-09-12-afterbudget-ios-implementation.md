# AfterBudget iOS Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Construire AfterBudget iOS (SwiftUI, offline, GRDB) avec parité desktop complète — onboarding, budget, transactions, catégories, statistiques, récurrences, réglages, import/export — et calculs de solde prévisionnel alignés sur la SPEC.

**Architecture:** Application SwiftUI iPhone-only (iOS 17+) organisée en features MVVM (`Features/<Name>/{Models,Repository,Service,ViewModels,Views}`) plus un noyau partagé (`Core/Domain`, `Core/Database`, `Core/DesignSystem`). Persistence 100 % locale via GRDB/SQLite (schéma aligné desktop), injection via `AppDependencyContainer`, shell `TabView` + onboarding en `fullScreenCover`. Aucun réseau.

**Tech Stack:** Swift 5.9+, SwiftUI, XcodeGen, SPM GRDB (~7.x), XCTest, iOS 17 Simulator.

## Global Constraints

- Location: afterbudget-ios/ under AfterBudget on Mac (paths in plan: afterbudget-ios/AfterBudget/...)
- iOS 17+, iPhone only, SwiftUI, MVVM by features + GRDB SQLite
- English code, French comments/docs
- Offline, no API, no network update module
- Full desktop parity: Onboarding, Budget, Transactions, Categories, Statistics, Recurrences, Settings, ImportExport
- Design: sober editorial charcoal/paper/matte copper — NOT Encre & Cuivre, NOT AI-generic
- Tests: 1 logic file → 1 test file; unit tests per feature
- Amounts Int64 cents
- projected = currentBalance + pendingIncome - pendingExpenses
- Healthy: projected >= 0; Attention: projected < 0 && projected >= -overdraft; Danger: projected < -overdraft
- Schema: categories, transactions, app_settings, recurring_rules, schema_migrations (align desktop)
- TabView: Accueil|Transactions|Stats|Réglages; onboarding fullScreenCover
- Bundle id: app.afterbudget.ios ; scheme AfterBudget

---

## File Structure (locked)

```
afterbudget-ios/
├── project.yml
├── .gitignore
├── AfterBudget/
│   ├── App/
│   │   ├── AfterBudgetApp.swift
│   │   ├── AppDependencyContainer.swift
│   │   └── RootView.swift
│   ├── Core/
│   │   ├── Domain/
│   │   │   ├── Money.swift
│   │   │   ├── TransactionKind.swift
│   │   │   ├── TransactionStatus.swift
│   │   │   ├── ThemeMode.swift
│   │   │   ├── AppError.swift
│   │   │   └── FinancialStatus.swift
│   │   ├── Database/
│   │   │   ├── DatabaseClient.swift
│   │   │   ├── Migrations.swift
│   │   │   └── SeedCategories.swift
│   │   └── DesignSystem/
│   │       ├── ColorTokens.swift
│   │       ├── TypographyTokens.swift
│   │       ├── SpacingTokens.swift
│   │       ├── ABAmountText.swift
│   │       └── ABStatusPill.swift
│   └── Features/
│       ├── Categories/
│       ├── Settings/
│       ├── Budget/
│       ├── Transactions/
│       ├── Onboarding/
│       ├── Statistics/
│       ├── Recurrences/
│       └── ImportExport/
├── AfterBudgetTests/
│   ├── Core/
│   ├── Categories/
│   ├── Settings/
│   ├── Budget/
│   ├── Transactions/
│   ├── Statistics/
│   ├── Recurrences/
│   ├── ImportExport/
│   └── App/
└── docs/
    ├── ARCHITECTURE.md
    ├── SPEC.md
    ├── DATABASE.md
    ├── DESIGN_SYSTEM.md
    └── DECISIONS.md
```

---

### Task 1: Scaffold XcodeGen + SPM GRDB + dossiers + gitignore

**Files:**
- Create: `afterbudget-ios/project.yml`
- Create: `afterbudget-ios/.gitignore`
- Create: `afterbudget-ios/AfterBudget/App/AfterBudgetApp.swift`
- Create: `afterbudget-ios/AfterBudget/App/RootView.swift`
- Create: `afterbudget-ios/AfterBudgetTests/SmokeTests.swift`

**Interfaces:**
- Consumes: rien
- Produces: schéma XcodeGen `AfterBudget` (bundle `app.afterbudget.ios`), target tests `AfterBudgetTests`, dossier sources prêt pour GRDB

- [ ] **Step 1: Créer `.gitignore`**

```gitignore
# afterbudget-ios/.gitignore
.DS_Store
xcuserdata/
*.xcuserstate
DerivedData/
.build/
*.xcodeproj/
*.xcworkspace/
!project.yml
.swiftpm/
Packages/
```

- [ ] **Step 2: Écrire `project.yml`**

```yaml
# afterbudget-ios/project.yml
name: AfterBudget
options:
  bundleIdPrefix: app.afterbudget
  deploymentTarget:
    iOS: "17.0"
  xcodeVersion: "16.0"
  createIntermediateGroups: true
settings:
  base:
    SWIFT_VERSION: "5.9"
    TARGETED_DEVICE_FAMILY: "1"
    IPHONEOS_DEPLOYMENT_TARGET: "17.0"
    PRODUCT_BUNDLE_IDENTIFIER: app.afterbudget.ios
packages:
  GRDB:
    url: https://github.com/groue/GRDB.swift
    from: "7.0.0"
targets:
  AfterBudget:
    type: application
    platform: iOS
    sources:
      - path: AfterBudget
    settings:
      base:
        PRODUCT_BUNDLE_IDENTIFIER: app.afterbudget.ios
        INFOPLIST_KEY_UIApplicationSceneManifest_Generation: YES
        INFOPLIST_KEY_UILaunchScreen_Generation: YES
        INFOPLIST_KEY_CFBundleDisplayName: AfterBudget
        INFOPLIST_KEY_UISupportedInterfaceOrientations: UIInterfaceOrientationPortrait
        GENERATE_INFOPLIST_FILE: YES
    dependencies:
      - package: GRDB
        product: GRDB
    scheme:
      testTargets:
        - AfterBudgetTests
  AfterBudgetTests:
    type: bundle.unit-test
    platform: iOS
    sources:
      - path: AfterBudgetTests
    dependencies:
      - target: AfterBudget
    settings:
      base:
        PRODUCT_BUNDLE_IDENTIFIER: app.afterbudget.ios.tests
        GENERATE_INFOPLIST_FILE: YES
```

- [ ] **Step 3: Stub app + smoke test**

```swift
// afterbudget-ios/AfterBudget/App/AfterBudgetApp.swift
import SwiftUI

@main
struct AfterBudgetApp: App {
    var body: some Scene {
        WindowGroup {
            RootView()
        }
    }
}
```

```swift
// afterbudget-ios/AfterBudget/App/RootView.swift
import SwiftUI

/// Racine temporaire — remplacée au Task 14.
struct RootView: View {
    var body: some View {
        Text("AfterBudget")
            .accessibilityIdentifier("root.placeholder")
    }
}
```

```swift
// afterbudget-ios/AfterBudgetTests/SmokeTests.swift
import XCTest
@testable import AfterBudget

final class SmokeTests: XCTestCase {
    func testBundleLoads() {
        XCTAssertTrue(true)
    }
}
```

- [ ] **Step 4: Générer le projet et lancer le smoke test**

Run:

```bash
cd afterbudget-ios && xcodegen generate && xcodebuild test -scheme AfterBudget -destination 'platform=iOS Simulator,name=iPhone 16' -only-testing:AfterBudgetTests/SmokeTests
```

Expected: `** TEST SUCCEEDED **`

- [ ] **Step 5: Commit**

```bash
cd afterbudget-ios && git init && git add project.yml .gitignore AfterBudget AfterBudgetTests && git commit -m "chore: scaffold AfterBudget iOS with XcodeGen and GRDB"
```

---

### Task 2: Core Domain — Money, kinds, status, theme, errors, FinancialStatus

**Files:**
- Create: `afterbudget-ios/AfterBudget/Core/Domain/Money.swift`
- Create: `afterbudget-ios/AfterBudget/Core/Domain/TransactionKind.swift`
- Create: `afterbudget-ios/AfterBudget/Core/Domain/TransactionStatus.swift`
- Create: `afterbudget-ios/AfterBudget/Core/Domain/ThemeMode.swift`
- Create: `afterbudget-ios/AfterBudget/Core/Domain/AppError.swift`
- Create: `afterbudget-ios/AfterBudget/Core/Domain/FinancialStatus.swift`
- Test: `afterbudget-ios/AfterBudgetTests/Core/MoneyTests.swift`
- Test: `afterbudget-ios/AfterBudgetTests/Core/FinancialStatusTests.swift`

**Interfaces:**
- Consumes: rien
- Produces:
  - `struct Money: Hashable, Comparable { var cents: Int64; static let zero; init(cents:); static func fromInput(_:); func formatFR(); checkedAdd/Sub }`
  - `enum TransactionKind: String { case income, expense }`
  - `enum TransactionStatus: String { case pending, completed }`
  - `enum ThemeMode: String { case system, light, dark }`
  - `enum AppError: Error, Equatable { case validation(String); case database(String); case notFound; case overflow }`
  - `enum FinancialStatus: Equatable { case healthy, attention, danger; static func resolve(projected:Money, overdraft:Money) -> FinancialStatus }`

- [ ] **Step 1: Write failing Money + FinancialStatus tests**

```swift
// afterbudget-ios/AfterBudgetTests/Core/MoneyTests.swift
import XCTest
@testable import AfterBudget

final class MoneyTests: XCTestCase {
    func testFromInputFrenchDecimal() throws {
        let m = try Money.fromInput("10,50")
        XCTAssertEqual(m.cents, 1050)
    }

    func testFromInputRejectsTooManyDecimals() {
        XCTAssertThrowsError(try Money.fromInput("10,123"))
    }

    func testFormatFR() {
        XCTAssertEqual(Money(cents: -47800).formatFR(), "-478,00 €")
        XCTAssertEqual(Money(cents: 1000).formatFR(), "10,00 €")
    }

    func testCheckedAddOverflow() {
        XCTAssertNil(Money(cents: Int64.max).checkedAdd(Money(cents: 1)))
    }
}
```

```swift
// afterbudget-ios/AfterBudgetTests/Core/FinancialStatusTests.swift
import XCTest
@testable import AfterBudget

final class FinancialStatusTests: XCTestCase {
    func testHealthyWhenProjectedNonNegative() {
        XCTAssertEqual(FinancialStatus.resolve(projected: Money(cents: 0), overdraft: Money(cents: 50_000)), .healthy)
    }

    func testAttentionInsideOverdraft() {
        XCTAssertEqual(FinancialStatus.resolve(projected: Money(cents: -47_800), overdraft: Money(cents: 50_000)), .attention)
    }

    func testDangerBeyondOverdraft() {
        XCTAssertEqual(FinancialStatus.resolve(projected: Money(cents: -70_000), overdraft: Money(cents: 50_000)), .danger)
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run:

```bash
cd afterbudget-ios && xcodegen generate && xcodebuild test -scheme AfterBudget -destination 'platform=iOS Simulator,name=iPhone 16' -only-testing:AfterBudgetTests/MoneyTests -only-testing:AfterBudgetTests/FinancialStatusTests
```

Expected: FAIL (types introuvables)

- [ ] **Step 3: Minimal domain implementation**

```swift
// afterbudget-ios/AfterBudget/Core/Domain/Money.swift
import Foundation

/// Montant en centimes (Int64) — jamais de Float/Double en stockage.
struct Money: Hashable, Comparable, Sendable {
    var cents: Int64
    static let zero = Money(cents: 0)
    init(cents: Int64) { self.cents = cents }
    static func < (lhs: Money, rhs: Money) -> Bool { lhs.cents < rhs.cents }

    static func fromInput(_ raw: String) throws -> Money {
        let cleaned = raw
            .replacingOccurrences(of: "\u{00a0}", with: "")
            .replacingOccurrences(of: "\u{202f}", with: "")
            .replacingOccurrences(of: " ", with: "")
            .replacingOccurrences(of: ",", with: ".")
            .trimmingCharacters(in: .whitespaces)
        guard !cleaned.isEmpty else { throw AppError.validation("Le montant est obligatoire.") }
        var negative = false
        var body = cleaned
        if body.hasPrefix("-") { negative = true; body.removeFirst() }
        else if body.hasPrefix("+") { body.removeFirst() }
        guard !body.isEmpty else { throw AppError.validation("Montant invalide : « \(raw) »") }
        let parts = body.split(separator: ".", omittingEmptySubsequences: false)
        guard parts.count <= 2 else { throw AppError.validation("Montant invalide : « \(raw) »") }
        let whole = String(parts[0])
        let frac = parts.count == 2 ? String(parts[1]) : ""
        guard whole.allSatisfy(\.isNumber), frac.allSatisfy(\.isNumber), frac.count <= 2 else {
            throw AppError.validation("Montant invalide : « \(raw) »")
        }
        guard let w = Int64(whole) else { throw AppError.validation("Le montant est trop grand.") }
        let f: Int64
        if frac.isEmpty { f = 0 }
        else if frac.count == 1 {
            guard let d = Int64(frac) else { throw AppError.validation("Le montant est trop grand.") }
            f = d * 10
        } else {
            guard let d = Int64(frac) else { throw AppError.validation("Le montant est trop grand.") }
            f = d
        }
        guard w <= (Int64.max - f) / 100 else { throw AppError.validation("Le montant est trop grand.") }
        var cents = w * 100 + f
        if negative {
            let (neg, overflow) = cents.multipliedReportingOverflow(by: -1)
            if overflow { throw AppError.overflow }
            cents = neg
        }
        return Money(cents: cents)
    }

    func formatFR(currencyCode: String = "EUR") -> String {
        let absCents = abs(cents)
        let whole = absCents / 100
        let frac = absCents % 100
        let sign = cents < 0 ? "-" : ""
        let symbol = currencyCode == "EUR" ? "€" : currencyCode
        return String(format: "%@%lld,%02lld %@", sign, whole, frac, symbol)
    }

    func checkedAdd(_ other: Money) -> Money? {
        let (r, o) = cents.addingReportingOverflow(other.cents)
        return o ? nil : Money(cents: r)
    }

    func checkedSub(_ other: Money) -> Money? {
        let (r, o) = cents.subtractingReportingOverflow(other.cents)
        return o ? nil : Money(cents: r)
    }

    var isNegative: Bool { cents < 0 }
}
```

```swift
// afterbudget-ios/AfterBudget/Core/Domain/TransactionKind.swift
import Foundation

enum TransactionKind: String, Codable, CaseIterable, Sendable {
    case income, expense
    var displayName: String { self == .income ? "Revenu" : "Dépense" }
}
```

```swift
// afterbudget-ios/AfterBudget/Core/Domain/TransactionStatus.swift
import Foundation

enum TransactionStatus: String, Codable, CaseIterable, Sendable {
    case pending, completed
    var displayName: String { self == .pending ? "En attente" : "Réalisé" }
}
```

```swift
// afterbudget-ios/AfterBudget/Core/Domain/ThemeMode.swift
import Foundation

enum ThemeMode: String, Codable, CaseIterable, Sendable {
    case system, light, dark
    var displayName: String {
        switch self {
        case .system: return "Système"
        case .light: return "Clair"
        case .dark: return "Sombre"
        }
    }
}
```

```swift
// afterbudget-ios/AfterBudget/Core/Domain/AppError.swift
import Foundation

enum AppError: Error, Equatable, LocalizedError {
    case validation(String)
    case database(String)
    case notFound
    case overflow
    var errorDescription: String? {
        switch self {
        case .validation(let m): return m
        case .database(let m): return m
        case .notFound: return "Élément introuvable."
        case .overflow: return "Dépassement de montant."
        }
    }
}
```

```swift
// afterbudget-ios/AfterBudget/Core/Domain/FinancialStatus.swift
import Foundation

/// Statut financier dérivé du solde prévisionnel et du découvert.
enum FinancialStatus: Equatable, Sendable {
    case healthy, attention, danger
    var displayName: String {
        switch self {
        case .healthy: return "En forme"
        case .attention: return "Attention"
        case .danger: return "Danger"
        }
    }
    static func resolve(projected: Money, overdraft: Money) -> FinancialStatus {
        if projected.cents >= 0 { return .healthy }
        if projected.cents >= -overdraft.cents { return .attention }
        return .danger
    }
    func message(margin: Money) -> String {
        switch self {
        case .healthy:
            return "Tu devrais terminer le mois avec un solde positif."
        case .attention:
            return "Tu devrais utiliser une partie de ton découvert. Il te reste \(margin.formatFR()) de marge."
        case .danger:
            return "Tu risques de dépasser ton découvert autorisé de \(Money(cents: -margin.cents).formatFR())."
        }
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run:

```bash
cd afterbudget-ios && xcodegen generate && xcodebuild test -scheme AfterBudget -destination 'platform=iOS Simulator,name=iPhone 16' -only-testing:AfterBudgetTests/MoneyTests -only-testing:AfterBudgetTests/FinancialStatusTests
```

Expected: PASS

- [ ] **Step 5: Commit**

```bash
cd afterbudget-ios && git add AfterBudget/Core/Domain AfterBudgetTests/Core && git commit -m "feat: add core domain Money, kinds, FinancialStatus"
```

---

### Task 3: DatabaseClient + migrations + tests in-memory

**Files:**
- Create: `afterbudget-ios/AfterBudget/Core/Database/DatabaseClient.swift`
- Create: `afterbudget-ios/AfterBudget/Core/Database/Migrations.swift`
- Create: `afterbudget-ios/AfterBudget/Core/Database/SeedCategories.swift`
- Test: `afterbudget-ios/AfterBudgetTests/Core/DatabaseClientTests.swift`

**Interfaces:**
- Consumes: `Money`, `TransactionKind`, `ThemeMode`
- Produces: `DatabaseClient.makeInMemory()` / `makeOnDisk(url:)` / `dbQueue: DatabaseQueue` ; `AppMigration.register` ; seed 26 expense + 9 income (ids desktop)

- [ ] **Step 1: Write failing migration test**

```swift
// afterbudget-ios/AfterBudgetTests/Core/DatabaseClientTests.swift
import XCTest
import GRDB
@testable import AfterBudget

final class DatabaseClientTests: XCTestCase {
    func testMigrationsCreateExpectedTablesAndSeed() throws {
        let client = try DatabaseClient.makeInMemory()
        try client.dbQueue.read { db in
            let tables = try String.fetchAll(db, sql: "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            for name in ["app_settings", "categories", "recurring_rules", "schema_migrations", "transactions"] {
                XCTAssertTrue(tables.contains(name), "missing \(name)")
            }
            XCTAssertEqual(try Int.fetchOne(db, sql: "SELECT COUNT(*) FROM categories"), 35)
            XCTAssertEqual(try Int.fetchOne(db, sql: "SELECT COUNT(*) FROM app_settings"), 1)
            XCTAssertEqual(try Int.fetchOne(db, sql: "SELECT MAX(version) FROM schema_migrations"), 3)
        }
    }

    func testForeignKeysEnabled() throws {
        let client = try DatabaseClient.makeInMemory()
        try client.dbQueue.read { db in
            XCTAssertEqual(try Bool.fetchOne(db, sql: "PRAGMA foreign_keys"), true)
        }
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```bash
cd afterbudget-ios && xcodegen generate && xcodebuild test -scheme AfterBudget -destination 'platform=iOS Simulator,name=iPhone 16' -only-testing:AfterBudgetTests/DatabaseClientTests
```

Expected: FAIL (`DatabaseClient` missing)

- [ ] **Step 3: Implement SeedCategories + Migrations + DatabaseClient**

```swift
// afterbudget-ios/AfterBudget/Core/Database/SeedCategories.swift
import Foundation

struct SeedCategory: Sendable {
    let id: String
    let kind: TransactionKind
    let name: String
    let icon: String
    let color: String
    let sortOrder: Int
}

enum SeedCategories {
    static let all: [SeedCategory] = expense + income
    static let expense: [SeedCategory] = [
        .init(id: "logement", kind: .expense, name: "Logement", icon: "Home", color: "#3B82F6", sortOrder: 1),
        .init(id: "alimentation", kind: .expense, name: "Alimentation", icon: "ShoppingCart", color: "#22C55E", sortOrder: 2),
        .init(id: "transport", kind: .expense, name: "Transport", icon: "Car", color: "#EAB308", sortOrder: 3),
        .init(id: "sante", kind: .expense, name: "Santé", icon: "Heart", color: "#EF4444", sortOrder: 4),
        .init(id: "loisirs", kind: .expense, name: "Loisirs", icon: "Gamepad2", color: "#A855F7", sortOrder: 5),
        .init(id: "vetements", kind: .expense, name: "Vêtements", icon: "Shirt", color: "#EC4899", sortOrder: 6),
        .init(id: "abonnements", kind: .expense, name: "Abonnements", icon: "RefreshCw", color: "#6366F1", sortOrder: 7),
        .init(id: "restaurants", kind: .expense, name: "Restaurants", icon: "Utensils", color: "#F97316", sortOrder: 8),
        .init(id: "voyages", kind: .expense, name: "Voyages", icon: "Plane", color: "#06B6D4", sortOrder: 9),
        .init(id: "education", kind: .expense, name: "Éducation", icon: "GraduationCap", color: "#14B8A6", sortOrder: 10),
        .init(id: "epargne", kind: .expense, name: "Épargne", icon: "PiggyBank", color: "#10B981", sortOrder: 11),
        .init(id: "energie", kind: .expense, name: "Énergie", icon: "Zap", color: "#F59E0B", sortOrder: 12),
        .init(id: "internet", kind: .expense, name: "Internet", icon: "Wifi", color: "#0EA5E9", sortOrder: 13),
        .init(id: "telephone", kind: .expense, name: "Téléphone", icon: "Smartphone", color: "#8B5CF6", sortOrder: 14),
        .init(id: "sport", kind: .expense, name: "Sport", icon: "Dumbbell", color: "#84CC16", sortOrder: 15),
        .init(id: "musique", kind: .expense, name: "Musique", icon: "Music", color: "#F43F5E", sortOrder: 16),
        .init(id: "livres", kind: .expense, name: "Livres", icon: "BookOpen", color: "#78716C", sortOrder: 17),
        .init(id: "enfants", kind: .expense, name: "Enfants", icon: "Baby", color: "#D946EF", sortOrder: 18),
        .init(id: "animaux", kind: .expense, name: "Animaux", icon: "PawPrint", color: "#CA8A04", sortOrder: 19),
        .init(id: "bricolage", kind: .expense, name: "Bricolage", icon: "Wrench", color: "#6B7280", sortOrder: 20),
        .init(id: "cadeaux", kind: .expense, name: "Cadeaux", icon: "Gift", color: "#F472B6", sortOrder: 21),
        .init(id: "bien-etre", kind: .expense, name: "Bien-être", icon: "Sparkles", color: "#C084FC", sortOrder: 22),
        .init(id: "credit", kind: .expense, name: "Crédit", icon: "CreditCard", color: "#FB923C", sortOrder: 23),
        .init(id: "tabac", kind: .expense, name: "Tabac", icon: "Cigarette", color: "#A8A29E", sortOrder: 24),
        .init(id: "assurance", kind: .expense, name: "Assurance", icon: "Shield", color: "#38BDF8", sortOrder: 25),
        .init(id: "autre", kind: .expense, name: "Autre", icon: "MoreHorizontal", color: "#9CA3AF", sortOrder: 26),
    ]
    static let income: [SeedCategory] = [
        .init(id: "salaire", kind: .income, name: "Salaire", icon: "Wallet", color: "#22C55E", sortOrder: 1),
        .init(id: "allocation", kind: .income, name: "Allocation", icon: "HandCoins", color: "#10B981", sortOrder: 2),
        .init(id: "prime", kind: .income, name: "Prime", icon: "Star", color: "#F59E0B", sortOrder: 3),
        .init(id: "remboursement", kind: .income, name: "Remboursement", icon: "Undo2", color: "#3B82F6", sortOrder: 4),
        .init(id: "vente", kind: .income, name: "Vente", icon: "Tag", color: "#EC4899", sortOrder: 5),
        .init(id: "pension", kind: .income, name: "Pension", icon: "HeartHandshake", color: "#A855F7", sortOrder: 6),
        .init(id: "freelance", kind: .income, name: "Revenu indépendant", icon: "Briefcase", color: "#6366F1", sortOrder: 7),
        .init(id: "interets", kind: .income, name: "Intérêts", icon: "TrendingUp", color: "#14B8A6", sortOrder: 8),
        .init(id: "autre-revenu", kind: .income, name: "Autre", icon: "MoreHorizontal", color: "#9CA3AF", sortOrder: 9),
    ]
}
```

```swift
// afterbudget-ios/AfterBudget/Core/Database/Migrations.swift
import Foundation
import GRDB

enum AppMigration {
    static func register(_ migrator: inout DatabaseMigrator) {
        migrator.registerMigration("v1") { db in
            try db.execute(sql: """
                CREATE TABLE categories (
                    id TEXT PRIMARY KEY,
                    kind TEXT NOT NULL CHECK(kind IN ('income', 'expense')),
                    name TEXT NOT NULL,
                    icon TEXT NOT NULL,
                    color TEXT NOT NULL CHECK(length(color) = 7 AND substr(color, 1, 1) = '#'),
                    sort_order INTEGER NOT NULL DEFAULT 0,
                    is_default INTEGER NOT NULL DEFAULT 1,
                    is_active INTEGER NOT NULL DEFAULT 1,
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL,
                    UNIQUE(kind, name)
                );
                CREATE TABLE transactions (
                    id TEXT PRIMARY KEY,
                    kind TEXT NOT NULL CHECK(kind IN ('income', 'expense')),
                    label TEXT NOT NULL,
                    amount_cents INTEGER NOT NULL CHECK(amount_cents > 0),
                    transaction_date TEXT NOT NULL,
                    status TEXT NOT NULL CHECK(status IN ('pending', 'completed')),
                    category_id TEXT NOT NULL REFERENCES categories(id),
                    note TEXT,
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL
                );
                CREATE TABLE app_settings (
                    id INTEGER PRIMARY KEY CHECK(id = 1),
                    current_balance_cents INTEGER NOT NULL DEFAULT 0,
                    overdraft_limit_cents INTEGER NOT NULL DEFAULT 0 CHECK(overdraft_limit_cents >= 0),
                    currency_code TEXT NOT NULL DEFAULT 'EUR',
                    locale TEXT NOT NULL DEFAULT 'fr-FR',
                    theme TEXT NOT NULL DEFAULT 'system' CHECK(theme IN ('system', 'light', 'dark')),
                    balance_updated_at TEXT NOT NULL,
                    onboarding_completed INTEGER NOT NULL DEFAULT 0,
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL
                );
                CREATE INDEX idx_transactions_date ON transactions(transaction_date);
                CREATE INDEX idx_transactions_kind_status ON transactions(kind, status);
                CREATE INDEX idx_transactions_category ON transactions(category_id);
                CREATE INDEX idx_categories_kind ON categories(kind);
                """)
            let now = ISO8601DateFormatter().string(from: Date())
            for cat in SeedCategories.all {
                try db.execute(
                    sql: """
                    INSERT OR IGNORE INTO categories
                    (id, kind, name, icon, color, sort_order, is_default, is_active, created_at, updated_at)
                    VALUES (?, ?, ?, ?, ?, ?, 1, 1, ?, ?)
                    """,
                    arguments: [cat.id, cat.kind.rawValue, cat.name, cat.icon, cat.color, cat.sortOrder, now, now]
                )
            }
            try db.execute(
                sql: """
                INSERT INTO app_settings
                (id, current_balance_cents, overdraft_limit_cents, currency_code, locale, theme,
                 balance_updated_at, onboarding_completed, created_at, updated_at)
                VALUES (1, 0, 0, 'EUR', 'fr-FR', 'system', ?, 0, ?, ?)
                """,
                arguments: [now, now, now]
            )
        }
        migrator.registerMigration("v2") { db in
            try db.execute(sql: """
                CREATE TABLE recurring_rules (
                    id TEXT PRIMARY KEY,
                    kind TEXT NOT NULL CHECK(kind IN ('income', 'expense')),
                    label TEXT NOT NULL,
                    amount_cents INTEGER NOT NULL CHECK(amount_cents > 0),
                    category_id TEXT NOT NULL REFERENCES categories(id),
                    day_of_month INTEGER NOT NULL CHECK(day_of_month BETWEEN 1 AND 31),
                    start_year INTEGER NOT NULL,
                    start_month INTEGER NOT NULL CHECK(start_month BETWEEN 1 AND 12),
                    end_year INTEGER,
                    end_month INTEGER CHECK(end_month IS NULL OR end_month BETWEEN 1 AND 12),
                    note TEXT,
                    is_active INTEGER NOT NULL DEFAULT 1,
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL
                );
                """)
            let columns = try db.columns(in: "transactions").map(\.name)
            if !columns.contains("recurring_rule_id") {
                try db.execute(sql: "ALTER TABLE transactions ADD COLUMN recurring_rule_id TEXT REFERENCES recurring_rules(id);")
            }
            try db.execute(sql: """
                CREATE INDEX IF NOT EXISTS idx_transactions_recurring ON transactions(recurring_rule_id);
                CREATE INDEX IF NOT EXISTS idx_recurring_active ON recurring_rules(is_active);
                """)
        }
        migrator.registerMigration("v3") { db in
            let columns = try db.columns(in: "app_settings").map(\.name)
            if !columns.contains("last_export_date") {
                try db.execute(sql: "ALTER TABLE app_settings ADD COLUMN last_export_date TEXT;")
            }
        }
    }
}
```

```swift
// afterbudget-ios/AfterBudget/Core/Database/DatabaseClient.swift
import Foundation
import GRDB

/// Client SQLite GRDB — offline uniquement.
final class DatabaseClient: Sendable {
    let dbQueue: DatabaseQueue

    private init(dbQueue: DatabaseQueue) throws {
        self.dbQueue = dbQueue
        var migrator = DatabaseMigrator()
        AppMigration.register(&migrator)
        try migrator.migrate(dbQueue)
        try dbQueue.write { db in
            try db.execute(sql: "PRAGMA foreign_keys = ON;")
            try db.execute(sql: """
                CREATE TABLE IF NOT EXISTS schema_migrations (
                    version INTEGER PRIMARY KEY,
                    name TEXT,
                    applied_at TEXT
                );
                """)
            let max = try Int.fetchOne(db, sql: "SELECT COALESCE(MAX(version), 0) FROM schema_migrations") ?? 0
            let now = ISO8601DateFormatter().string(from: Date())
            if max < 1 { try db.execute(sql: "INSERT INTO schema_migrations VALUES (1, 'v1', ?)", arguments: [now]) }
            if max < 2 { try db.execute(sql: "INSERT INTO schema_migrations VALUES (2, 'v2', ?)", arguments: [now]) }
            if max < 3 { try db.execute(sql: "INSERT INTO schema_migrations VALUES (3, 'v3', ?)", arguments: [now]) }
        }
    }

    static func makeInMemory() throws -> DatabaseClient {
        var config = Configuration()
        config.foreignKeysEnabled = true
        return try DatabaseClient(dbQueue: DatabaseQueue(configuration: config))
    }

    static func makeOnDisk(url: URL) throws -> DatabaseClient {
        var config = Configuration()
        config.foreignKeysEnabled = true
        return try DatabaseClient(dbQueue: DatabaseQueue(path: url.path, configuration: config))
    }

    static func defaultDatabaseURL() throws -> URL {
        let fm = FileManager.default
        let dir = try fm.url(for: .applicationSupportDirectory, in: .userDomainMask, appropriateFor: nil, create: true)
            .appendingPathComponent("afterbudget", isDirectory: true)
        try fm.createDirectory(at: dir, withIntermediateDirectories: true)
        return dir.appendingPathComponent("afterbudget.sqlite")
    }
}
```

- [ ] **Step 4: Run tests — pass**

Run:

```bash
cd afterbudget-ios && xcodegen generate && xcodebuild test -scheme AfterBudget -destination 'platform=iOS Simulator,name=iPhone 16' -only-testing:AfterBudgetTests/DatabaseClientTests
```

Expected: PASS

- [ ] **Step 5: Commit**

```bash
cd afterbudget-ios && git add AfterBudget/Core/Database AfterBudgetTests/Core/DatabaseClientTests.swift && git commit -m "feat: add GRDB DatabaseClient with desktop-aligned migrations"
```

---

### Task 4: DesignSystem tokens + ABAmountText + ABStatusPill

**Files:**
- Create: `afterbudget-ios/AfterBudget/Core/DesignSystem/ColorTokens.swift`
- Create: `afterbudget-ios/AfterBudget/Core/DesignSystem/TypographyTokens.swift`
- Create: `afterbudget-ios/AfterBudget/Core/DesignSystem/SpacingTokens.swift`
- Create: `afterbudget-ios/AfterBudget/Core/DesignSystem/ABAmountText.swift`
- Create: `afterbudget-ios/AfterBudget/Core/DesignSystem/ABStatusPill.swift`
- Test: `afterbudget-ios/AfterBudgetTests/Core/DesignSystemTests.swift`

**Interfaces:**
- Consumes: `Money`, `FinancialStatus`
- Produces: tokens charcoal/paper/matte copper (≠ Encre & Cuivre) ; `ABAmountText` ; `ABStatusPill`

- [ ] **Step 1: Write failing design tests**

```swift
// afterbudget-ios/AfterBudgetTests/Core/DesignSystemTests.swift
import XCTest
@testable import AfterBudget

final class DesignSystemTests: XCTestCase {
    func testPaletteIsNotEncreEtCuivre() {
        XCTAssertNotEqual(ColorTokens.Light.paperHex, "#F4F0E9")
        XCTAssertNotEqual(ColorTokens.Light.accentHex, "#9C5A2E")
        XCTAssertNotEqual(ColorTokens.Dark.charcoalHex, "#14110D")
        XCTAssertEqual(ColorTokens.Light.paperHex, "#F7F5F1")
        XCTAssertEqual(ColorTokens.Light.accentHex, "#8B6F47")
    }

    func testStatusPillLabels() {
        XCTAssertEqual(ABStatusPill.label(for: .healthy), "En forme")
        XCTAssertEqual(ABStatusPill.label(for: .attention), "Attention")
        XCTAssertEqual(ABStatusPill.label(for: .danger), "Danger")
    }
}
```

- [ ] **Step 2: Run — expect FAIL**

Run:

```bash
cd afterbudget-ios && xcodegen generate && xcodebuild test -scheme AfterBudget -destination 'platform=iOS Simulator,name=iPhone 16' -only-testing:AfterBudgetTests/DesignSystemTests
```

Expected: FAIL

- [ ] **Step 3: Implement tokens + components**

```swift
// afterbudget-ios/AfterBudget/Core/DesignSystem/ColorTokens.swift
import SwiftUI

/// Direction visuelle : éditorial sobre charcoal / paper / matte copper.
/// Intentionnellement distincte d'« Encre & Cuivre » desktop.
enum ColorTokens {
    enum Light {
        static let paperHex = "#F7F5F1"
        static let charcoalHex = "#1C1B19"
        static let accentHex = "#8B6F47"
        static let surfaceHex = "#FFFDF9"
        static let borderHex = "#E4DFD6"
        static let mutedHex = "#6F6A62"
        static let incomeHex = "#2F6B4F"
        static let expenseHex = "#9B3A3A"
        static let healthyHex = "#2F6B4F"
        static let attentionHex = "#8A6A1F"
        static let dangerHex = "#9B3A3A"
    }
    enum Dark {
        static let charcoalHex = "#121110"
        static let paperHex = "#EDE9E2"
        static let accentHex = "#C4A574"
        static let surfaceHex = "#1C1A18"
        static let borderHex = "#2E2B27"
        static let mutedHex = "#A39E94"
        static let incomeHex = "#7DB89A"
        static let expenseHex = "#E09090"
        static let healthyHex = "#7DB89A"
        static let attentionHex = "#D4B56A"
        static let dangerHex = "#E09090"
    }
    static func hex(_ value: String) -> Color {
        let h = value.trimmingCharacters(in: CharacterSet(charactersIn: "#"))
        let n = UInt64(h, radix: 16) ?? 0
        return Color(red: Double((n >> 16) & 0xFF) / 255, green: Double((n >> 8) & 0xFF) / 255, blue: Double(n & 0xFF) / 255)
    }
}
```

```swift
// afterbudget-ios/AfterBudget/Core/DesignSystem/TypographyTokens.swift
import SwiftUI

enum TypographyTokens {
    static let display = Font.system(size: 40, weight: .semibold, design: .serif)
    static let title = Font.system(size: 22, weight: .semibold, design: .default)
    static let body = Font.system(size: 16, weight: .regular, design: .default)
    static let caption = Font.system(size: 13, weight: .regular, design: .default)
    static let monoAmount = Font.system(size: 17, weight: .medium, design: .rounded)
}
```

```swift
// afterbudget-ios/AfterBudget/Core/DesignSystem/SpacingTokens.swift
import CoreGraphics

enum SpacingTokens {
    static let xs: CGFloat = 4
    static let sm: CGFloat = 8
    static let md: CGFloat = 16
    static let lg: CGFloat = 24
    static let xl: CGFloat = 32
}
```

```swift
// afterbudget-ios/AfterBudget/Core/DesignSystem/ABAmountText.swift
import SwiftUI

struct ABAmountText: View {
    let money: Money
    var kind: TransactionKind? = nil
    var body: some View {
        Text(display).font(TypographyTokens.monoAmount).foregroundStyle(color).accessibilityLabel(money.formatFR())
    }
    private var display: String {
        guard let kind else { return money.formatFR() }
        let prefix = kind == .income ? "+" : "−"
        return prefix + Money(cents: abs(money.cents)).formatFR()
    }
    private var color: Color {
        switch kind {
        case .income: return ColorTokens.hex(ColorTokens.Light.incomeHex)
        case .expense: return ColorTokens.hex(ColorTokens.Light.expenseHex)
        case nil: return ColorTokens.hex(ColorTokens.Light.charcoalHex)
        }
    }
}
```

```swift
// afterbudget-ios/AfterBudget/Core/DesignSystem/ABStatusPill.swift
import SwiftUI

struct ABStatusPill: View {
    let status: FinancialStatus
    var body: some View {
        Text(Self.label(for: status))
            .font(TypographyTokens.caption.weight(.semibold))
            .padding(.horizontal, SpacingTokens.sm)
            .padding(.vertical, SpacingTokens.xs)
            .background(background)
            .foregroundStyle(foreground)
            .clipShape(Capsule())
            .accessibilityLabel(Self.label(for: status))
    }
    static func label(for status: FinancialStatus) -> String { status.displayName }
    private var foreground: Color {
        switch status {
        case .healthy: return ColorTokens.hex(ColorTokens.Light.healthyHex)
        case .attention: return ColorTokens.hex(ColorTokens.Light.attentionHex)
        case .danger: return ColorTokens.hex(ColorTokens.Light.dangerHex)
        }
    }
    private var background: Color { foreground.opacity(0.14) }
}
```

- [ ] **Step 4: Run — PASS**

Run:

```bash
cd afterbudget-ios && xcodegen generate && xcodebuild test -scheme AfterBudget -destination 'platform=iOS Simulator,name=iPhone 16' -only-testing:AfterBudgetTests/DesignSystemTests
```

Expected: PASS

- [ ] **Step 5: Commit**

```bash
cd afterbudget-ios && git add AfterBudget/Core/DesignSystem AfterBudgetTests/Core/DesignSystemTests.swift && git commit -m "feat: add charcoal/paper/matte-copper design system"
```

---

### Task 5: Categories models / repo / service + tests

**Files:**
- Create: `afterbudget-ios/AfterBudget/Features/Categories/Models/Category.swift`
- Create: `afterbudget-ios/AfterBudget/Features/Categories/Repository/CategoryRepository.swift`
- Create: `afterbudget-ios/AfterBudget/Features/Categories/Service/CategoryService.swift`
- Test: `afterbudget-ios/AfterBudgetTests/Categories/CategoryServiceTests.swift`

**Interfaces:**
- Consumes: `DatabaseClient`, `TransactionKind`
- Produces: `Category` ; `CategoryRepository.fetchAll/fetchActive/fetch` ; `CategoryService.listActive/require`

- [ ] **Step 1: Write failing test**

```swift
// afterbudget-ios/AfterBudgetTests/Categories/CategoryServiceTests.swift
import XCTest
@testable import AfterBudget

final class CategoryServiceTests: XCTestCase {
    func testListsSeededExpenseCategories() throws {
        let db = try DatabaseClient.makeInMemory()
        let service = CategoryService(repository: CategoryRepository(db: db))
        let expenses = try service.listActive(kind: .expense)
        XCTAssertEqual(expenses.count, 26)
        XCTAssertEqual(expenses.first?.id, "logement")
    }

    func testRequireRejectsWrongKind() throws {
        let db = try DatabaseClient.makeInMemory()
        let service = CategoryService(repository: CategoryRepository(db: db))
        XCTAssertThrowsError(try service.require(id: "salaire", kind: .expense))
    }
}
```

- [ ] **Step 2: Run — FAIL**

Run:

```bash
cd afterbudget-ios && xcodegen generate && xcodebuild test -scheme AfterBudget -destination 'platform=iOS Simulator,name=iPhone 16' -only-testing:AfterBudgetTests/CategoryServiceTests
```

Expected: FAIL

- [ ] **Step 3: Implement**

```swift
// afterbudget-ios/AfterBudget/Features/Categories/Models/Category.swift
import Foundation

struct Category: Identifiable, Equatable, Sendable {
    let id: String
    let kind: TransactionKind
    let name: String
    let icon: String
    let color: String
    let sortOrder: Int
    let isDefault: Bool
    let isActive: Bool
}
```

```swift
// afterbudget-ios/AfterBudget/Features/Categories/Repository/CategoryRepository.swift
import Foundation
import GRDB

struct CategoryRepository: Sendable {
    let db: DatabaseClient
    func fetchAll() throws -> [Category] {
        try db.dbQueue.read { db in
            try Row.fetchAll(db, sql: "SELECT * FROM categories ORDER BY kind, sort_order, name").map(Self.map)
        }
    }
    func fetchActive(kind: TransactionKind) throws -> [Category] {
        try db.dbQueue.read { db in
            try Row.fetchAll(
                db,
                sql: "SELECT * FROM categories WHERE kind = ? AND is_active = 1 ORDER BY sort_order, name",
                arguments: [kind.rawValue]
            ).map(Self.map)
        }
    }
    func fetch(id: String) throws -> Category? {
        try db.dbQueue.read { db in
            try Row.fetchOne(db, sql: "SELECT * FROM categories WHERE id = ?", arguments: [id]).map(Self.map)
        }
    }
    private static func map(_ row: Row) -> Category {
        Category(
            id: row["id"],
            kind: TransactionKind(rawValue: row["kind"]) ?? .expense,
            name: row["name"],
            icon: row["icon"],
            color: row["color"],
            sortOrder: row["sort_order"],
            isDefault: (row["is_default"] as Int) == 1,
            isActive: (row["is_active"] as Int) == 1
        )
    }
}
```

```swift
// afterbudget-ios/AfterBudget/Features/Categories/Service/CategoryService.swift
import Foundation

struct CategoryService: Sendable {
    let repository: CategoryRepository
    func listActive(kind: TransactionKind) throws -> [Category] { try repository.fetchActive(kind: kind) }
    func require(id: String, kind: TransactionKind) throws -> Category {
        guard let cat = try repository.fetch(id: id) else { throw AppError.notFound }
        guard cat.kind == kind, cat.isActive else {
            throw AppError.validation("Catégorie invalide pour ce type d'opération.")
        }
        return cat
    }
}
```

- [ ] **Step 4: Run — PASS**

Run:

```bash
cd afterbudget-ios && xcodegen generate && xcodebuild test -scheme AfterBudget -destination 'platform=iOS Simulator,name=iPhone 16' -only-testing:AfterBudgetTests/CategoryServiceTests
```

Expected: PASS

- [ ] **Step 5: Commit**

```bash
cd afterbudget-ios && git add AfterBudget/Features/Categories AfterBudgetTests/Categories && git commit -m "feat: add categories repository and service"
```

---

### Task 6: Settings models / DTOs / repo / service + tests

**Files:**
- Create: `afterbudget-ios/AfterBudget/Features/Settings/Models/AppSettings.swift`
- Create: `afterbudget-ios/AfterBudget/Features/Settings/DTOs/UpdateSettingsDTO.swift`
- Create: `afterbudget-ios/AfterBudget/Features/Settings/Repository/SettingsRepository.swift`
- Create: `afterbudget-ios/AfterBudget/Features/Settings/Service/SettingsService.swift`
- Test: `afterbudget-ios/AfterBudgetTests/Settings/SettingsServiceTests.swift`

**Interfaces:**
- Consumes: `DatabaseClient`, `Money`, `ThemeMode`
- Produces: `AppSettings` ; `UpdateSettingsDTO` ; `SettingsService.fetch/update/completeOnboarding/resetAll/markExported`

- [ ] **Step 1: Write failing test**

```swift
// afterbudget-ios/AfterBudgetTests/Settings/SettingsServiceTests.swift
import XCTest
@testable import AfterBudget

final class SettingsServiceTests: XCTestCase {
    func testDefaultSettings() throws {
        let service = SettingsService(repository: SettingsRepository(db: try DatabaseClient.makeInMemory()))
        let s = try service.fetch()
        XCTAssertEqual(s.currentBalance.cents, 0)
        XCTAssertFalse(s.onboardingCompleted)
        XCTAssertEqual(s.theme, .system)
    }

    func testCompleteOnboarding() throws {
        let service = SettingsService(repository: SettingsRepository(db: try DatabaseClient.makeInMemory()))
        try service.completeOnboarding(balance: Money(cents: 12_500), overdraft: Money(cents: 50_000))
        let s = try service.fetch()
        XCTAssertTrue(s.onboardingCompleted)
        XCTAssertEqual(s.currentBalance.cents, 12_500)
        XCTAssertEqual(s.overdraftLimit.cents, 50_000)
    }

    func testRejectsNegativeOverdraft() throws {
        let service = SettingsService(repository: SettingsRepository(db: try DatabaseClient.makeInMemory()))
        XCTAssertThrowsError(try service.update(UpdateSettingsDTO(currentBalanceCents: 0, overdraftLimitCents: -1, theme: .light)))
    }
}
```

- [ ] **Step 2: Run — FAIL**

Run:

```bash
cd afterbudget-ios && xcodegen generate && xcodebuild test -scheme AfterBudget -destination 'platform=iOS Simulator,name=iPhone 16' -only-testing:AfterBudgetTests/SettingsServiceTests
```

Expected: FAIL

- [ ] **Step 3: Implement**

```swift
// afterbudget-ios/AfterBudget/Features/Settings/Models/AppSettings.swift
import Foundation

struct AppSettings: Equatable, Sendable {
    var currentBalance: Money
    var overdraftLimit: Money
    var currencyCode: String
    var locale: String
    var theme: ThemeMode
    var balanceUpdatedAt: String
    var onboardingCompleted: Bool
    var lastExportDate: String?
}
```

```swift
// afterbudget-ios/AfterBudget/Features/Settings/DTOs/UpdateSettingsDTO.swift
import Foundation

struct UpdateSettingsDTO: Equatable, Sendable {
    var currentBalanceCents: Int64
    var overdraftLimitCents: Int64
    var theme: ThemeMode
}
```

```swift
// afterbudget-ios/AfterBudget/Features/Settings/Repository/SettingsRepository.swift
import Foundation
import GRDB

struct SettingsRepository: Sendable {
    let db: DatabaseClient

    func fetch() throws -> AppSettings {
        try db.dbQueue.read { db in
            guard let row = try Row.fetchOne(db, sql: "SELECT * FROM app_settings WHERE id = 1") else {
                throw AppError.database("Paramètres introuvables.")
            }
            return AppSettings(
                currentBalance: Money(cents: row["current_balance_cents"]),
                overdraftLimit: Money(cents: row["overdraft_limit_cents"]),
                currencyCode: row["currency_code"],
                locale: row["locale"],
                theme: ThemeMode(rawValue: row["theme"]) ?? .system,
                balanceUpdatedAt: row["balance_updated_at"],
                onboardingCompleted: (row["onboarding_completed"] as Int) == 1,
                lastExportDate: row["last_export_date"]
            )
        }
    }

    func update(balance: Money, overdraft: Money, theme: ThemeMode, onboardingCompleted: Bool?, lastExportDate: String?) throws {
        let now = ISO8601DateFormatter().string(from: Date())
        try db.dbQueue.write { db in
            if let onboardingCompleted {
                try db.execute(
                    sql: """
                    UPDATE app_settings SET current_balance_cents=?, overdraft_limit_cents=?, theme=?,
                    balance_updated_at=?, onboarding_completed=?, last_export_date=COALESCE(?, last_export_date), updated_at=?
                    WHERE id=1
                    """,
                    arguments: [balance.cents, overdraft.cents, theme.rawValue, now, onboardingCompleted ? 1 : 0, lastExportDate, now]
                )
            } else {
                try db.execute(
                    sql: """
                    UPDATE app_settings SET current_balance_cents=?, overdraft_limit_cents=?, theme=?,
                    balance_updated_at=?, last_export_date=COALESCE(?, last_export_date), updated_at=?
                    WHERE id=1
                    """,
                    arguments: [balance.cents, overdraft.cents, theme.rawValue, now, lastExportDate, now]
                )
            }
        }
    }

    func resetTransactionalData() throws {
        try db.dbQueue.write { db in
            try db.execute(sql: "DELETE FROM transactions;")
            try db.execute(sql: "DELETE FROM recurring_rules;")
            let now = ISO8601DateFormatter().string(from: Date())
            try db.execute(
                sql: """
                UPDATE app_settings SET current_balance_cents=0, overdraft_limit_cents=0, theme='system',
                onboarding_completed=0, last_export_date=NULL, balance_updated_at=?, updated_at=? WHERE id=1
                """,
                arguments: [now, now]
            )
        }
    }
}
```

```swift
// afterbudget-ios/AfterBudget/Features/Settings/Service/SettingsService.swift
import Foundation

struct SettingsService: Sendable {
    let repository: SettingsRepository
    func fetch() throws -> AppSettings { try repository.fetch() }
    func update(_ dto: UpdateSettingsDTO) throws {
        guard dto.overdraftLimitCents >= 0 else {
            throw AppError.validation("Le découvert autorisé doit être positif ou nul.")
        }
        try repository.update(
            balance: Money(cents: dto.currentBalanceCents),
            overdraft: Money(cents: dto.overdraftLimitCents),
            theme: dto.theme,
            onboardingCompleted: nil,
            lastExportDate: nil
        )
    }
    func completeOnboarding(balance: Money, overdraft: Money) throws {
        guard overdraft.cents >= 0 else {
            throw AppError.validation("Le découvert autorisé doit être positif ou nul.")
        }
        try repository.update(balance: balance, overdraft: overdraft, theme: .system, onboardingCompleted: true, lastExportDate: nil)
    }
    func markExported(at isoDate: String) throws {
        let current = try repository.fetch()
        try repository.update(balance: current.currentBalance, overdraft: current.overdraftLimit, theme: current.theme, onboardingCompleted: nil, lastExportDate: isoDate)
    }
    func resetAll() throws { try repository.resetTransactionalData() }
}
```

- [ ] **Step 4: Run — PASS**

Run:

```bash
cd afterbudget-ios && xcodegen generate && xcodebuild test -scheme AfterBudget -destination 'platform=iOS Simulator,name=iPhone 16' -only-testing:AfterBudgetTests/SettingsServiceTests
```

Expected: PASS

- [ ] **Step 5: Commit**

```bash
cd afterbudget-ios && git add AfterBudget/Features/Settings AfterBudgetTests/Settings && git commit -m "feat: add settings models, repository and service"
```

---

### Task 7: BudgetCalculator pure + tests SPEC (−478 Attention, 600 Healthy, −700 Danger)

**Files:**
- Create: `afterbudget-ios/AfterBudget/Features/Budget/Models/BudgetSummary.swift`
- Create: `afterbudget-ios/AfterBudget/Features/Budget/Service/BudgetCalculator.swift`
- Test: `afterbudget-ios/AfterBudgetTests/Budget/BudgetCalculatorTests.swift`

**Interfaces:**
- Consumes: `Money`, `FinancialStatus`
- Produces: `BudgetSummary` ; `BudgetCalculator.compute(...) throws -> BudgetSummary` ; `projected = currentBalance + pendingIncome - pendingExpenses`

- [ ] **Step 1: Write failing SPEC tests**

```swift
// afterbudget-ios/AfterBudgetTests/Budget/BudgetCalculatorTests.swift
import XCTest
@testable import AfterBudget

final class BudgetCalculatorTests: XCTestCase {
    /// SPEC 22.1 — Attention (−478 €, découvert 500 €)
    func testSpecAttentionMinus478() throws {
        let summary = try BudgetCalculator.compute(
            currentBalance: Money(cents: -36_000),
            overdraftLimit: Money(cents: 50_000),
            pendingIncome: Money(cents: 120_700),
            pendingExpenses: Money(cents: 132_500),
            totalIncome: Money(cents: 120_700),
            totalExpenses: Money(cents: 132_500),
            completedIncome: .zero,
            completedExpenses: .zero
        )
        XCTAssertEqual(summary.projectedBalance.cents, -47_800)
        XCTAssertEqual(summary.remainingOverdraftMargin.cents, 2_200)
        XCTAssertEqual(summary.status, .attention)
    }

    /// SPEC 22.2 — Healthy (600 €)
    func testSpecHealthy600() throws {
        let summary = try BudgetCalculator.compute(
            currentBalance: Money(cents: 30_000),
            overdraftLimit: Money(cents: 20_000),
            pendingIncome: Money(cents: 150_000),
            pendingExpenses: Money(cents: 120_000),
            totalIncome: Money(cents: 150_000),
            totalExpenses: Money(cents: 120_000),
            completedIncome: .zero,
            completedExpenses: .zero
        )
        XCTAssertEqual(summary.projectedBalance.cents, 60_000)
        XCTAssertEqual(summary.status, .healthy)
    }

    /// SPEC 22.3 — Danger (−700 €)
    func testSpecDangerMinus700() throws {
        let summary = try BudgetCalculator.compute(
            currentBalance: Money(cents: -40_000),
            overdraftLimit: Money(cents: 50_000),
            pendingIncome: Money(cents: 20_000),
            pendingExpenses: Money(cents: 50_000),
            totalIncome: Money(cents: 20_000),
            totalExpenses: Money(cents: 50_000),
            completedIncome: .zero,
            completedExpenses: .zero
        )
        XCTAssertEqual(summary.projectedBalance.cents, -70_000)
        XCTAssertEqual(summary.remainingOverdraftMargin.cents, -20_000)
        XCTAssertEqual(summary.status, .danger)
    }
}
```

- [ ] **Step 2: Run — FAIL**

Run:

```bash
cd afterbudget-ios && xcodegen generate && xcodebuild test -scheme AfterBudget -destination 'platform=iOS Simulator,name=iPhone 16' -only-testing:AfterBudgetTests/BudgetCalculatorTests
```

Expected: FAIL

- [ ] **Step 3: Implement calculator**

```swift
// afterbudget-ios/AfterBudget/Features/Budget/Models/BudgetSummary.swift
import Foundation

struct BudgetSummary: Equatable, Sendable {
    var currentBalance: Money
    var pendingIncome: Money
    var pendingExpenses: Money
    var projectedBalance: Money
    var overdraftLimit: Money
    var remainingOverdraftMargin: Money
    var status: FinancialStatus
    var totalIncome: Money
    var totalExpenses: Money
    var completedIncome: Money
    var completedExpenses: Money

    var remainingOverdraft: Money {
        if projectedBalance.isNegative {
            return Money(cents: max(0, overdraftLimit.cents + projectedBalance.cents))
        }
        return overdraftLimit
    }

    var overdraftUsed: Money {
        if projectedBalance.isNegative {
            return Money(cents: min(overdraftLimit.cents, -projectedBalance.cents))
        }
        return .zero
    }

    var overdraftOverage: Money {
        Money(cents: max(0, -remainingOverdraftMargin.cents))
    }
}
```

```swift
// afterbudget-ios/AfterBudget/Features/Budget/Service/BudgetCalculator.swift
import Foundation

enum BudgetCalculator {
    static func compute(
        currentBalance: Money,
        overdraftLimit: Money,
        pendingIncome: Money,
        pendingExpenses: Money,
        totalIncome: Money,
        totalExpenses: Money,
        completedIncome: Money,
        completedExpenses: Money
    ) throws -> BudgetSummary {
        guard let projected = currentBalance.checkedAdd(pendingIncome)?.checkedSub(pendingExpenses) else {
            throw AppError.overflow
        }
        guard let margin = projected.checkedAdd(overdraftLimit) else { throw AppError.overflow }
        return BudgetSummary(
            currentBalance: currentBalance,
            pendingIncome: pendingIncome,
            pendingExpenses: pendingExpenses,
            projectedBalance: projected,
            overdraftLimit: overdraftLimit,
            remainingOverdraftMargin: margin,
            status: FinancialStatus.resolve(projected: projected, overdraft: overdraftLimit),
            totalIncome: totalIncome,
            totalExpenses: totalExpenses,
            completedIncome: completedIncome,
            completedExpenses: completedExpenses
        )
    }
}
```

- [ ] **Step 4: Run — PASS**

Run:

```bash
cd afterbudget-ios && xcodegen generate && xcodebuild test -scheme AfterBudget -destination 'platform=iOS Simulator,name=iPhone 16' -only-testing:AfterBudgetTests/BudgetCalculatorTests
```

Expected: PASS

- [ ] **Step 5: Commit**

```bash
cd afterbudget-ios && git add AfterBudget/Features/Budget AfterBudgetTests/Budget && git commit -m "feat: add BudgetCalculator with SPEC examples"
```

---

### Task 8: Transactions DTOs / validators / repo / service / list ViewModel + tests

**Files:**
- Create: `afterbudget-ios/AfterBudget/Features/Transactions/Models/Transaction.swift`
- Create: `afterbudget-ios/AfterBudget/Features/Transactions/DTOs/CreateTransactionDTO.swift`
- Create: `afterbudget-ios/AfterBudget/Features/Transactions/DTOs/UpdateTransactionDTO.swift`
- Create: `afterbudget-ios/AfterBudget/Features/Transactions/DTOs/FilterTransactionsDTO.swift`
- Create: `afterbudget-ios/AfterBudget/Features/Transactions/Validators/TransactionValidator.swift`
- Create: `afterbudget-ios/AfterBudget/Features/Transactions/Repository/TransactionRepository.swift`
- Create: `afterbudget-ios/AfterBudget/Features/Transactions/Service/TransactionService.swift`
- Create: `afterbudget-ios/AfterBudget/Features/Transactions/ViewModels/TransactionListViewModel.swift`
- Create: `afterbudget-ios/AfterBudget/Features/Transactions/Views/TransactionListView.swift`
- Test: `afterbudget-ios/AfterBudgetTests/Transactions/TransactionServiceTests.swift`

**Interfaces:**
- Consumes: `DatabaseClient`, `CategoryService`, `Money`, `TransactionKind`, `TransactionStatus`
- Produces: `Transaction` ; `MonthSums` ; `TransactionService.create/update/delete/list/sumsForMonth` ; `TransactionListViewModel`

- [ ] **Step 1: Write failing service tests**

```swift
// afterbudget-ios/AfterBudgetTests/Transactions/TransactionServiceTests.swift
import XCTest
@testable import AfterBudget

final class TransactionServiceTests: XCTestCase {
    private func makeService() throws -> TransactionService {
        let db = try DatabaseClient.makeInMemory()
        return TransactionService(
            repository: TransactionRepository(db: db),
            categories: CategoryService(repository: CategoryRepository(db: db))
        )
    }

    func testCreatePendingExpense() throws {
        let service = try makeService()
        let tx = try service.create(CreateTransactionDTO(
            kind: .expense, label: "Courses", amountCents: 2_450,
            transactionDate: "2026-09-12", status: .pending, categoryId: "alimentation", note: nil
        ))
        XCTAssertEqual(tx.amount.cents, 2_450)
        XCTAssertEqual(tx.status, .pending)
        XCTAssertNil(tx.recurringRuleId)
    }

    func testRejectsZeroAmount() throws {
        let service = try makeService()
        XCTAssertThrowsError(try service.create(CreateTransactionDTO(
            kind: .income, label: "X", amountCents: 0,
            transactionDate: "2026-09-12", status: .completed, categoryId: "salaire", note: nil
        )))
    }

    func testMonthSumsPendingOnly() throws {
        let service = try makeService()
        _ = try service.create(CreateTransactionDTO(
            kind: .expense, label: "A", amountCents: 1000,
            transactionDate: "2026-09-01", status: .pending, categoryId: "autre", note: nil
        ))
        _ = try service.create(CreateTransactionDTO(
            kind: .expense, label: "B", amountCents: 2000,
            transactionDate: "2026-09-02", status: .completed, categoryId: "autre", note: nil
        ))
        let sums = try service.sumsForMonth(year: 2026, month: 9)
        XCTAssertEqual(sums.pendingExpenses.cents, 1000)
        XCTAssertEqual(sums.totalExpenses.cents, 3000)
    }

    func testUpdateAndDelete() throws {
        let service = try makeService()
        let created = try service.create(CreateTransactionDTO(
            kind: .income, label: "Prime", amountCents: 5000,
            transactionDate: "2026-09-15", status: .pending, categoryId: "prime", note: nil
        ))
        let updated = try service.update(UpdateTransactionDTO(
            id: created.id, kind: .income, label: "Prime OK", amountCents: 6000,
            transactionDate: "2026-09-15", status: .completed, categoryId: "prime", note: "n"
        ))
        XCTAssertEqual(updated.amount.cents, 6000)
        XCTAssertEqual(updated.status, .completed)
        try service.delete(id: created.id)
        XCTAssertTrue(try service.list(FilterTransactionsDTO(year: 2026, month: 9)).isEmpty)
    }
}
```

- [ ] **Step 2: Run — FAIL**

Run:

```bash
cd afterbudget-ios && xcodegen generate && xcodebuild test -scheme AfterBudget -destination 'platform=iOS Simulator,name=iPhone 16' -only-testing:AfterBudgetTests/TransactionServiceTests
```

Expected: FAIL

- [ ] **Step 3: Implement transactions stack**

```swift
// afterbudget-ios/AfterBudget/Features/Transactions/Models/Transaction.swift
import Foundation

struct Transaction: Identifiable, Equatable, Sendable {
    let id: String
    var kind: TransactionKind
    var label: String
    var amount: Money
    var transactionDate: String
    var status: TransactionStatus
    var categoryId: String
    var note: String?
    var recurringRuleId: String?
    var createdAt: String
    var updatedAt: String
}
```

```swift
// afterbudget-ios/AfterBudget/Features/Transactions/DTOs/CreateTransactionDTO.swift
import Foundation

struct CreateTransactionDTO: Equatable, Sendable {
    var kind: TransactionKind
    var label: String
    var amountCents: Int64
    var transactionDate: String
    var status: TransactionStatus
    var categoryId: String
    var note: String?
}
```

```swift
// afterbudget-ios/AfterBudget/Features/Transactions/DTOs/UpdateTransactionDTO.swift
import Foundation

struct UpdateTransactionDTO: Equatable, Sendable {
    var id: String
    var kind: TransactionKind
    var label: String
    var amountCents: Int64
    var transactionDate: String
    var status: TransactionStatus
    var categoryId: String
    var note: String?
}
```

```swift
// afterbudget-ios/AfterBudget/Features/Transactions/DTOs/FilterTransactionsDTO.swift
import Foundation

struct FilterTransactionsDTO: Equatable, Sendable {
    var year: Int
    var month: Int
    var kind: TransactionKind? = nil
    var status: TransactionStatus? = nil
    var categoryId: String? = nil
    var search: String? = nil
}
```

```swift
// afterbudget-ios/AfterBudget/Features/Transactions/Validators/TransactionValidator.swift
import Foundation

enum TransactionValidator {
    static func validateLabel(_ label: String) throws -> String {
        let t = label.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !t.isEmpty else { throw AppError.validation("Le libellé est obligatoire.") }
        guard t.count <= 120 else { throw AppError.validation("Le libellé est trop long.") }
        return t
    }
    static func validateAmountCents(_ cents: Int64) throws -> Money {
        guard cents > 0 else { throw AppError.validation("Le montant doit être strictement positif.") }
        return Money(cents: cents)
    }
    static func validateDate(_ raw: String) throws -> String {
        let parts = raw.split(separator: "-")
        guard parts.count == 3,
              let y = Int(parts[0]), let m = Int(parts[1]), let d = Int(parts[2]),
              (1...12).contains(m), (1...31).contains(d), y >= 2000 else {
            throw AppError.validation("Date invalide (YYYY-MM-DD).")
        }
        return raw
    }
}
```

```swift
// afterbudget-ios/AfterBudget/Features/Transactions/Repository/TransactionRepository.swift
import Foundation
import GRDB

struct MonthSums: Equatable, Sendable {
    var pendingIncome: Money
    var pendingExpenses: Money
    var totalIncome: Money
    var totalExpenses: Money
}

struct TransactionRepository: Sendable {
    let db: DatabaseClient

    func insert(_ tx: Transaction) throws {
        try db.dbQueue.write { db in
            try db.execute(
                sql: """
                INSERT INTO transactions
                (id, kind, label, amount_cents, transaction_date, status, category_id, note,
                 recurring_rule_id, created_at, updated_at)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                """,
                arguments: [
                    tx.id, tx.kind.rawValue, tx.label, tx.amount.cents, tx.transactionDate,
                    tx.status.rawValue, tx.categoryId, tx.note, tx.recurringRuleId, tx.createdAt, tx.updatedAt
                ]
            )
        }
    }

    func update(_ tx: Transaction) throws {
        try db.dbQueue.write { db in
            try db.execute(
                sql: """
                UPDATE transactions SET kind=?, label=?, amount_cents=?, transaction_date=?, status=?,
                category_id=?, note=?, updated_at=? WHERE id=?
                """,
                arguments: [
                    tx.kind.rawValue, tx.label, tx.amount.cents, tx.transactionDate,
                    tx.status.rawValue, tx.categoryId, tx.note, tx.updatedAt, tx.id
                ]
            )
        }
    }

    func delete(id: String) throws {
        try db.dbQueue.write { db in
            try db.execute(sql: "DELETE FROM transactions WHERE id = ?", arguments: [id])
        }
    }

    func fetchById(_ id: String) throws -> Transaction? {
        try db.dbQueue.read { db in
            try Row.fetchOne(db, sql: "SELECT * FROM transactions WHERE id = ?", arguments: [id]).map(Self.map)
        }
    }

    func fetch(filter: FilterTransactionsDTO) throws -> [Transaction] {
        let prefix = String(format: "%04d-%02d", filter.year, filter.month)
        var sql = "SELECT * FROM transactions WHERE substr(transaction_date,1,7) = ?"
        var args: [any DatabaseValueConvertible] = [prefix]
        if let kind = filter.kind { sql += " AND kind = ?"; args.append(kind.rawValue) }
        if let status = filter.status { sql += " AND status = ?"; args.append(status.rawValue) }
        if let categoryId = filter.categoryId { sql += " AND category_id = ?"; args.append(categoryId) }
        if let search = filter.search?.trimmingCharacters(in: .whitespaces), !search.isEmpty {
            sql += " AND (label LIKE ? OR IFNULL(note,'') LIKE ?)"
            args.append("%\(search)%"); args.append("%\(search)%")
        }
        sql += " ORDER BY transaction_date DESC, created_at DESC"
        return try db.dbQueue.read { db in
            try Row.fetchAll(db, sql: sql, arguments: StatementArguments(args)).map(Self.map)
        }
    }

    func sumsForMonth(year: Int, month: Int) throws -> MonthSums {
        let prefix = String(format: "%04d-%02d", year, month)
        return try db.dbQueue.read { db in
            func sum(kind: String, status: String?) throws -> Int64 {
                if let status {
                    return try Int64.fetchOne(db, sql: """
                        SELECT COALESCE(SUM(amount_cents),0) FROM transactions
                        WHERE substr(transaction_date,1,7)=? AND kind=? AND status=?
                        """, arguments: [prefix, kind, status]) ?? 0
                }
                return try Int64.fetchOne(db, sql: """
                    SELECT COALESCE(SUM(amount_cents),0) FROM transactions
                    WHERE substr(transaction_date,1,7)=? AND kind=?
                    """, arguments: [prefix, kind]) ?? 0
            }
            return MonthSums(
                pendingIncome: Money(cents: try sum(kind: "income", status: "pending")),
                pendingExpenses: Money(cents: try sum(kind: "expense", status: "pending")),
                totalIncome: Money(cents: try sum(kind: "income", status: nil)),
                totalExpenses: Money(cents: try sum(kind: "expense", status: nil))
            )
        }
    }

    private static func map(_ row: Row) -> Transaction {
        Transaction(
            id: row["id"],
            kind: TransactionKind(rawValue: row["kind"]) ?? .expense,
            label: row["label"],
            amount: Money(cents: row["amount_cents"]),
            transactionDate: row["transaction_date"],
            status: TransactionStatus(rawValue: row["status"]) ?? .pending,
            categoryId: row["category_id"],
            note: row["note"],
            recurringRuleId: row["recurring_rule_id"],
            createdAt: row["created_at"],
            updatedAt: row["updated_at"]
        )
    }
}
```

```swift
// afterbudget-ios/AfterBudget/Features/Transactions/Service/TransactionService.swift
import Foundation

struct TransactionService: Sendable {
    let repository: TransactionRepository
    let categories: CategoryService

    func create(_ dto: CreateTransactionDTO) throws -> Transaction {
        let label = try TransactionValidator.validateLabel(dto.label)
        let amount = try TransactionValidator.validateAmountCents(dto.amountCents)
        let date = try TransactionValidator.validateDate(dto.transactionDate)
        _ = try categories.require(id: dto.categoryId, kind: dto.kind)
        let now = ISO8601DateFormatter().string(from: Date())
        let tx = Transaction(
            id: UUID().uuidString.lowercased(), kind: dto.kind, label: label, amount: amount,
            transactionDate: date, status: dto.status, categoryId: dto.categoryId, note: dto.note,
            recurringRuleId: nil, createdAt: now, updatedAt: now
        )
        try repository.insert(tx)
        return tx
    }

    func update(_ dto: UpdateTransactionDTO) throws -> Transaction {
        let label = try TransactionValidator.validateLabel(dto.label)
        let amount = try TransactionValidator.validateAmountCents(dto.amountCents)
        let date = try TransactionValidator.validateDate(dto.transactionDate)
        _ = try categories.require(id: dto.categoryId, kind: dto.kind)
        guard var tx = try repository.fetchById(dto.id) else { throw AppError.notFound }
        let now = ISO8601DateFormatter().string(from: Date())
        tx.kind = dto.kind; tx.label = label; tx.amount = amount; tx.transactionDate = date
        tx.status = dto.status; tx.categoryId = dto.categoryId; tx.note = dto.note; tx.updatedAt = now
        try repository.update(tx)
        return tx
    }

    func delete(id: String) throws { try repository.delete(id: id) }
    func list(_ filter: FilterTransactionsDTO) throws -> [Transaction] { try repository.fetch(filter: filter) }
    func sumsForMonth(year: Int, month: Int) throws -> MonthSums { try repository.sumsForMonth(year: year, month: month) }
}
```

```swift
// afterbudget-ios/AfterBudget/Features/Transactions/ViewModels/TransactionListViewModel.swift
import Foundation

@MainActor
final class TransactionListViewModel: ObservableObject {
    @Published var items: [Transaction] = []
    @Published var filter: FilterTransactionsDTO
    @Published var errorMessage: String?
    private let service: TransactionService
    init(service: TransactionService, year: Int, month: Int) {
        self.service = service
        self.filter = FilterTransactionsDTO(year: year, month: month)
    }
    func reload() {
        do { items = try service.list(filter); errorMessage = nil }
        catch { errorMessage = error.localizedDescription }
    }
    func delete(id: String) {
        do { try service.delete(id: id); reload() }
        catch { errorMessage = error.localizedDescription }
    }
}
```

```swift
// afterbudget-ios/AfterBudget/Features/Transactions/Views/TransactionListView.swift
import SwiftUI

struct TransactionListView: View {
    @ObservedObject var viewModel: TransactionListViewModel
    var body: some View {
        NavigationStack {
            List(viewModel.items) { tx in
                HStack {
                    VStack(alignment: .leading, spacing: SpacingTokens.xs) {
                        Text(tx.label).font(TypographyTokens.body)
                        Text(tx.transactionDate).font(TypographyTokens.caption)
                            .foregroundStyle(ColorTokens.hex(ColorTokens.Light.mutedHex))
                    }
                    Spacer()
                    ABAmountText(money: tx.amount, kind: tx.kind)
                }
                .listRowBackground(ColorTokens.hex(ColorTokens.Light.surfaceHex))
            }
            .background(ColorTokens.hex(ColorTokens.Light.paperHex))
            .navigationTitle("Transactions")
            .onAppear { viewModel.reload() }
        }
    }
}
```

- [ ] **Step 4: Run — PASS**

Run:

```bash
cd afterbudget-ios && xcodegen generate && xcodebuild test -scheme AfterBudget -destination 'platform=iOS Simulator,name=iPhone 16' -only-testing:AfterBudgetTests/TransactionServiceTests
```

Expected: PASS

- [ ] **Step 5: Commit**

```bash
cd afterbudget-ios && git add AfterBudget/Features/Transactions AfterBudgetTests/Transactions && git commit -m "feat: add transactions CRUD, filters and list ViewModel"
```

---

### Task 9: Budget dashboard ViewModel + View

**Files:**
- Create: `afterbudget-ios/AfterBudget/Features/Budget/Service/BudgetService.swift`
- Create: `afterbudget-ios/AfterBudget/Features/Budget/ViewModels/BudgetDashboardViewModel.swift`
- Create: `afterbudget-ios/AfterBudget/Features/Budget/Views/BudgetDashboardView.swift`
- Test: `afterbudget-ios/AfterBudgetTests/Budget/BudgetServiceTests.swift`

**Interfaces:**
- Consumes: `SettingsService`, `TransactionService`, `BudgetCalculator`, `RecurrenceMaterializing?`
- Produces: `BudgetService.summary(year:month:)` ; `BudgetDashboardViewModel` ; `BudgetDashboardView`

- [ ] **Step 1: Write failing BudgetService test**

```swift
// afterbudget-ios/AfterBudgetTests/Budget/BudgetServiceTests.swift
import XCTest
@testable import AfterBudget

final class BudgetServiceTests: XCTestCase {
    func testSummaryUsesPendingOnly() throws {
        let db = try DatabaseClient.makeInMemory()
        let settings = SettingsService(repository: SettingsRepository(db: db))
        let categories = CategoryService(repository: CategoryRepository(db: db))
        let transactions = TransactionService(repository: TransactionRepository(db: db), categories: categories)
        try settings.completeOnboarding(balance: Money(cents: 30_000), overdraft: Money(cents: 20_000))
        _ = try transactions.create(CreateTransactionDTO(
            kind: .income, label: "Salaire", amountCents: 150_000,
            transactionDate: "2026-09-28", status: .pending, categoryId: "salaire", note: nil
        ))
        _ = try transactions.create(CreateTransactionDTO(
            kind: .expense, label: "Loyer", amountCents: 120_000,
            transactionDate: "2026-09-03", status: .pending, categoryId: "logement", note: nil
        ))
        let budget = BudgetService(settings: settings, transactions: transactions, recurrences: nil)
        let summary = try budget.summary(year: 2026, month: 9)
        XCTAssertEqual(summary.projectedBalance.cents, 60_000)
        XCTAssertEqual(summary.status, .healthy)
    }
}
```

- [ ] **Step 2: Run — FAIL**

Run:

```bash
cd afterbudget-ios && xcodegen generate && xcodebuild test -scheme AfterBudget -destination 'platform=iOS Simulator,name=iPhone 16' -only-testing:AfterBudgetTests/BudgetServiceTests
```

Expected: FAIL

- [ ] **Step 3: Implement BudgetService + VM + View**

```swift
// afterbudget-ios/AfterBudget/Features/Budget/Service/BudgetService.swift
import Foundation

protocol RecurrenceMaterializing: Sendable {
    func materialize(year: Int, month: Int) throws -> Int
}

struct BudgetService: Sendable {
    let settings: SettingsService
    let transactions: TransactionService
    let recurrences: (any RecurrenceMaterializing)?

    func summary(year: Int, month: Int) throws -> BudgetSummary {
        _ = try recurrences?.materialize(year: year, month: month)
        let s = try settings.fetch()
        let sums = try transactions.sumsForMonth(year: year, month: month)
        let completedIncome = sums.totalIncome.checkedSub(sums.pendingIncome) ?? .zero
        let completedExpenses = sums.totalExpenses.checkedSub(sums.pendingExpenses) ?? .zero
        return try BudgetCalculator.compute(
            currentBalance: s.currentBalance,
            overdraftLimit: s.overdraftLimit,
            pendingIncome: sums.pendingIncome,
            pendingExpenses: sums.pendingExpenses,
            totalIncome: sums.totalIncome,
            totalExpenses: sums.totalExpenses,
            completedIncome: completedIncome,
            completedExpenses: completedExpenses
        )
    }
}
```

```swift
// afterbudget-ios/AfterBudget/Features/Budget/ViewModels/BudgetDashboardViewModel.swift
import Foundation

@MainActor
final class BudgetDashboardViewModel: ObservableObject {
    @Published var summary: BudgetSummary?
    @Published var year: Int
    @Published var month: Int
    @Published var errorMessage: String?
    private let budget: BudgetService
    init(budget: BudgetService, year: Int, month: Int) {
        self.budget = budget; self.year = year; self.month = month
    }
    func reload() {
        do { summary = try budget.summary(year: year, month: month); errorMessage = nil }
        catch { errorMessage = error.localizedDescription }
    }
    func shiftMonth(_ delta: Int) {
        var m = month + delta; var y = year
        while m < 1 { m += 12; y -= 1 }
        while m > 12 { m -= 12; y += 1 }
        month = m; year = y; reload()
    }
}
```

```swift
// afterbudget-ios/AfterBudget/Features/Budget/Views/BudgetDashboardView.swift
import SwiftUI

struct BudgetDashboardView: View {
    @ObservedObject var viewModel: BudgetDashboardViewModel
    var body: some View {
        NavigationStack {
            ScrollView {
                if let summary = viewModel.summary {
                    VStack(alignment: .leading, spacing: SpacingTokens.lg) {
                        Text("Solde prévisionnel").font(TypographyTokens.caption)
                            .foregroundStyle(ColorTokens.hex(ColorTokens.Light.mutedHex))
                        Text(summary.projectedBalance.formatFR()).font(TypographyTokens.display)
                            .foregroundStyle(ColorTokens.hex(ColorTokens.Light.charcoalHex))
                        ABStatusPill(status: summary.status)
                        Text(summary.status.message(margin: summary.remainingOverdraft)).font(TypographyTokens.body)
                        HStack {
                            metric("À encaisser", summary.pendingIncome, .income)
                            metric("À payer", summary.pendingExpenses, .expense)
                        }
                    }
                    .padding(SpacingTokens.lg)
                }
            }
            .background(ColorTokens.hex(ColorTokens.Light.paperHex).ignoresSafeArea())
            .navigationTitle("Accueil")
            .toolbar {
                ToolbarItem(placement: .topBarLeading) { Button("Mois −") { viewModel.shiftMonth(-1) } }
                ToolbarItem(placement: .topBarTrailing) { Button("Mois +") { viewModel.shiftMonth(1) } }
            }
            .onAppear { viewModel.reload() }
        }
    }
    private func metric(_ title: String, _ money: Money, _ kind: TransactionKind) -> some View {
        VStack(alignment: .leading) {
            Text(title).font(TypographyTokens.caption)
            ABAmountText(money: money, kind: kind)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(SpacingTokens.md)
        .background(ColorTokens.hex(ColorTokens.Light.surfaceHex))
        .clipShape(RoundedRectangle(cornerRadius: 12))
    }
}
```

- [ ] **Step 4: Run — PASS**

Run:

```bash
cd afterbudget-ios && xcodegen generate && xcodebuild test -scheme AfterBudget -destination 'platform=iOS Simulator,name=iPhone 16' -only-testing:AfterBudgetTests/BudgetServiceTests
```

Expected: PASS

- [ ] **Step 5: Commit**

```bash
cd afterbudget-ios && git add AfterBudget/Features/Budget AfterBudgetTests/Budget && git commit -m "feat: add budget dashboard service, ViewModel and view"
```

---

### Task 10: Onboarding E2E

**Files:**
- Create: `afterbudget-ios/AfterBudget/Features/Onboarding/DTOs/CompleteOnboardingDTO.swift`
- Create: `afterbudget-ios/AfterBudget/Features/Onboarding/ViewModels/OnboardingViewModel.swift`
- Create: `afterbudget-ios/AfterBudget/Features/Onboarding/Views/OnboardingView.swift`
- Test: `afterbudget-ios/AfterBudgetTests/Settings/OnboardingViewModelTests.swift`

**Interfaces:**
- Consumes: `SettingsService`, `Money.fromInput`
- Produces: onboarding → `completeOnboarding` ; `onboardingCompleted == true`

- [ ] **Step 1: Write failing test**

```swift
// afterbudget-ios/AfterBudgetTests/Settings/OnboardingViewModelTests.swift
import XCTest
@testable import AfterBudget

@MainActor
final class OnboardingViewModelTests: XCTestCase {
    func testCompletesOnboardingWithParsedAmounts() throws {
        let settings = SettingsService(repository: SettingsRepository(db: try DatabaseClient.makeInMemory()))
        let vm = OnboardingViewModel(settings: settings)
        vm.balanceInput = "-360,00"
        vm.overdraftInput = "500"
        try vm.submit()
        let s = try settings.fetch()
        XCTAssertTrue(s.onboardingCompleted)
        XCTAssertEqual(s.currentBalance.cents, -36_000)
        XCTAssertEqual(s.overdraftLimit.cents, 50_000)
    }
}
```

- [ ] **Step 2: Run — FAIL**

Run:

```bash
cd afterbudget-ios && xcodegen generate && xcodebuild test -scheme AfterBudget -destination 'platform=iOS Simulator,name=iPhone 16' -only-testing:AfterBudgetTests/OnboardingViewModelTests
```

Expected: FAIL

- [ ] **Step 3: Implement**

```swift
// afterbudget-ios/AfterBudget/Features/Onboarding/DTOs/CompleteOnboardingDTO.swift
import Foundation

struct CompleteOnboardingDTO: Equatable, Sendable {
    var balanceCents: Int64
    var overdraftCents: Int64
}
```

```swift
// afterbudget-ios/AfterBudget/Features/Onboarding/ViewModels/OnboardingViewModel.swift
import Foundation

@MainActor
final class OnboardingViewModel: ObservableObject {
    @Published var step: Int = 0
    @Published var balanceInput: String = "0"
    @Published var overdraftInput: String = "0"
    @Published var errorMessage: String?
    private let settings: SettingsService
    init(settings: SettingsService) { self.settings = settings }
    func submit() throws {
        let balance = try Money.fromInput(balanceInput)
        let overdraft = try Money.fromInput(overdraftInput)
        guard overdraft.cents >= 0 else {
            throw AppError.validation("Le découvert autorisé doit être positif ou nul.")
        }
        try settings.completeOnboarding(balance: balance, overdraft: overdraft)
        errorMessage = nil
    }
}
```

```swift
// afterbudget-ios/AfterBudget/Features/Onboarding/Views/OnboardingView.swift
import SwiftUI

struct OnboardingView: View {
    @ObservedObject var viewModel: OnboardingViewModel
    var onFinished: () -> Void
    var body: some View {
        VStack(spacing: SpacingTokens.lg) {
            Text("AfterBudget").font(TypographyTokens.title)
            Text("Indique ton solde actuel et ton découvert autorisé.")
                .font(TypographyTokens.body).multilineTextAlignment(.center)
            TextField("Solde actuel", text: $viewModel.balanceInput)
                .keyboardType(.decimalPad).textFieldStyle(.roundedBorder)
            TextField("Découvert autorisé", text: $viewModel.overdraftInput)
                .keyboardType(.decimalPad).textFieldStyle(.roundedBorder)
            if let errorMessage = viewModel.errorMessage {
                Text(errorMessage).foregroundStyle(.red)
            }
            Button("Commencer") {
                do { try viewModel.submit(); onFinished() }
                catch { viewModel.errorMessage = error.localizedDescription }
            }
            .buttonStyle(.borderedProminent)
            .tint(ColorTokens.hex(ColorTokens.Light.accentHex))
        }
        .padding(SpacingTokens.xl)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background(ColorTokens.hex(ColorTokens.Light.paperHex).ignoresSafeArea())
    }
}
```

- [ ] **Step 4: Run — PASS**

Run:

```bash
cd afterbudget-ios && xcodegen generate && xcodebuild test -scheme AfterBudget -destination 'platform=iOS Simulator,name=iPhone 16' -only-testing:AfterBudgetTests/OnboardingViewModelTests
```

Expected: PASS

- [ ] **Step 5: Commit**

```bash
cd afterbudget-ios && git add AfterBudget/Features/Onboarding AfterBudgetTests/Settings/OnboardingViewModelTests.swift && git commit -m "feat: add onboarding flow ViewModel and view"
```

---

### Task 11: Statistics service + ViewModel + tests

**Files:**
- Create: `afterbudget-ios/AfterBudget/Features/Statistics/Models/MonthlyStatistics.swift`
- Create: `afterbudget-ios/AfterBudget/Features/Statistics/Service/StatisticsService.swift`
- Create: `afterbudget-ios/AfterBudget/Features/Statistics/ViewModels/StatisticsViewModel.swift`
- Create: `afterbudget-ios/AfterBudget/Features/Statistics/Views/StatisticsView.swift`
- Test: `afterbudget-ios/AfterBudgetTests/Statistics/StatisticsServiceTests.swift`

**Interfaces:**
- Consumes: `TransactionRepository`, `CategoryRepository`
- Produces: `MonthlyStatistics` + `CategoryStats` triés desc

- [ ] **Step 1: Write failing test**

```swift
// afterbudget-ios/AfterBudgetTests/Statistics/StatisticsServiceTests.swift
import XCTest
@testable import AfterBudget

final class StatisticsServiceTests: XCTestCase {
    func testBuildsCategoryBreakdown() throws {
        let db = try DatabaseClient.makeInMemory()
        let categories = CategoryService(repository: CategoryRepository(db: db))
        let tx = TransactionService(repository: TransactionRepository(db: db), categories: categories)
        _ = try tx.create(CreateTransactionDTO(
            kind: .expense, label: "Loyer", amountCents: 80_000,
            transactionDate: "2026-09-01", status: .completed, categoryId: "logement", note: nil
        ))
        _ = try tx.create(CreateTransactionDTO(
            kind: .expense, label: "Courses", amountCents: 20_000,
            transactionDate: "2026-09-05", status: .pending, categoryId: "alimentation", note: nil
        ))
        let service = StatisticsService(transactions: TransactionRepository(db: db), categories: CategoryRepository(db: db))
        let stats = try service.monthly(year: 2026, month: 9)
        XCTAssertEqual(stats.totalExpenses.cents, 100_000)
        XCTAssertEqual(stats.expensesByCategory.first?.categoryId, "logement")
        XCTAssertEqual(stats.expensesByCategory.first?.percentage ?? 0, 80, accuracy: 0.01)
    }
}
```

- [ ] **Step 2: Run — FAIL**

Run:

```bash
cd afterbudget-ios && xcodegen generate && xcodebuild test -scheme AfterBudget -destination 'platform=iOS Simulator,name=iPhone 16' -only-testing:AfterBudgetTests/StatisticsServiceTests
```

Expected: FAIL

- [ ] **Step 3: Implement**

```swift
// afterbudget-ios/AfterBudget/Features/Statistics/Models/MonthlyStatistics.swift
import Foundation

struct CategoryStats: Equatable, Identifiable, Sendable {
    var id: String { categoryId }
    var categoryId: String
    var categoryName: String
    var categoryColor: String
    var total: Money
    var percentage: Double
    var count: Int
}

struct MonthlyStatistics: Equatable, Sendable {
    var totalIncome: Money
    var totalExpenses: Money
    var balance: Money
    var completedIncome: Money
    var pendingIncome: Money
    var completedExpenses: Money
    var pendingExpenses: Money
    var transactionCount: Int
    var incomeCount: Int
    var expenseCount: Int
    var expensesByCategory: [CategoryStats]
    var incomeByCategory: [CategoryStats]
}
```

```swift
// afterbudget-ios/AfterBudget/Features/Statistics/Service/StatisticsService.swift
import Foundation

struct StatisticsService: Sendable {
    let transactions: TransactionRepository
    let categories: CategoryRepository

    func monthly(year: Int, month: Int) throws -> MonthlyStatistics {
        let sums = try transactions.sumsForMonth(year: year, month: month)
        let completedIncome = sums.totalIncome.checkedSub(sums.pendingIncome) ?? .zero
        let completedExpenses = sums.totalExpenses.checkedSub(sums.pendingExpenses) ?? .zero
        let balance = sums.totalIncome.checkedSub(sums.totalExpenses) ?? .zero
        let items = try transactions.fetch(filter: FilterTransactionsDTO(year: year, month: month))
        let incomeCount = items.filter { $0.kind == .income }.count
        let expenseCount = items.filter { $0.kind == .expense }.count
        return MonthlyStatistics(
            totalIncome: sums.totalIncome, totalExpenses: sums.totalExpenses, balance: balance,
            completedIncome: completedIncome, pendingIncome: sums.pendingIncome,
            completedExpenses: completedExpenses, pendingExpenses: sums.pendingExpenses,
            transactionCount: items.count, incomeCount: incomeCount, expenseCount: expenseCount,
            expensesByCategory: try byCategory(items: items, kind: .expense, grandTotal: sums.totalExpenses),
            incomeByCategory: try byCategory(items: items, kind: .income, grandTotal: sums.totalIncome)
        )
    }

    private func byCategory(items: [Transaction], kind: TransactionKind, grandTotal: Money) throws -> [CategoryStats] {
        let cats = try categories.fetchAll()
        let grouped = Dictionary(grouping: items.filter { $0.kind == kind }, by: \.categoryId)
        let grand = Double(grandTotal.cents)
        var result: [CategoryStats] = grouped.compactMap { id, list in
            guard let cat = cats.first(where: { $0.id == id }) else { return nil }
            let totalCents = list.reduce(Int64(0)) { $0 + $1.amount.cents }
            let pct = grand > 0 ? (Double(totalCents) / grand) * 100.0 : 0
            return CategoryStats(categoryId: id, categoryName: cat.name, categoryColor: cat.color,
                                 total: Money(cents: totalCents), percentage: pct, count: list.count)
        }
        result.sort { $0.total.cents > $1.total.cents }
        return result
    }
}
```

```swift
// afterbudget-ios/AfterBudget/Features/Statistics/ViewModels/StatisticsViewModel.swift
import Foundation

@MainActor
final class StatisticsViewModel: ObservableObject {
    @Published var stats: MonthlyStatistics?
    @Published var year: Int
    @Published var month: Int
    @Published var errorMessage: String?
    private let service: StatisticsService
    init(service: StatisticsService, year: Int, month: Int) {
        self.service = service; self.year = year; self.month = month
    }
    func reload() {
        do { stats = try service.monthly(year: year, month: month); errorMessage = nil }
        catch { errorMessage = error.localizedDescription }
    }
}
```

```swift
// afterbudget-ios/AfterBudget/Features/Statistics/Views/StatisticsView.swift
import SwiftUI

struct StatisticsView: View {
    @ObservedObject var viewModel: StatisticsViewModel
    var body: some View {
        NavigationStack {
            List {
                if let stats = viewModel.stats {
                    Section("Synthèse") {
                        LabeledContent("Revenus", value: stats.totalIncome.formatFR())
                        LabeledContent("Dépenses", value: stats.totalExpenses.formatFR())
                        LabeledContent("Solde mois", value: stats.balance.formatFR())
                    }
                    Section("Dépenses par catégorie") {
                        ForEach(stats.expensesByCategory) { row in
                            HStack {
                                Circle().fill(ColorTokens.hex(row.categoryColor)).frame(width: 10, height: 10)
                                Text(row.categoryName)
                                Spacer()
                                Text(String(format: "%.0f%%", row.percentage))
                                ABAmountText(money: row.total, kind: .expense)
                            }
                        }
                    }
                }
            }
            .navigationTitle("Stats")
            .onAppear { viewModel.reload() }
        }
    }
}
```

- [ ] **Step 4: Run — PASS**

Run:

```bash
cd afterbudget-ios && xcodegen generate && xcodebuild test -scheme AfterBudget -destination 'platform=iOS Simulator,name=iPhone 16' -only-testing:AfterBudgetTests/StatisticsServiceTests
```

Expected: PASS

- [ ] **Step 5: Commit**

```bash
cd afterbudget-ios && git add AfterBudget/Features/Statistics AfterBudgetTests/Statistics && git commit -m "feat: add monthly statistics service and views"
```

---

### Task 12: Recurrences materialize-for-month + tests

**Files:**
- Create: `afterbudget-ios/AfterBudget/Features/Recurrences/Models/RecurringRule.swift`
- Create: `afterbudget-ios/AfterBudget/Features/Recurrences/DTOs/CreateRecurrenceDTO.swift`
- Create: `afterbudget-ios/AfterBudget/Features/Recurrences/Repository/RecurrenceRepository.swift`
- Create: `afterbudget-ios/AfterBudget/Features/Recurrences/Service/RecurrenceService.swift`
- Test: `afterbudget-ios/AfterBudgetTests/Recurrences/RecurrenceServiceTests.swift`

**Interfaces:**
- Consumes: `DatabaseClient`, `CategoryService`, `TransactionKind`
- Produces: `RecurrenceService: RecurrenceMaterializing` ; `materialize(year:month:) -> Int` idempotent ; jour 31 → dernier jour du mois

- [ ] **Step 1: Write failing tests**

```swift
// afterbudget-ios/AfterBudgetTests/Recurrences/RecurrenceServiceTests.swift
import XCTest
@testable import AfterBudget

final class RecurrenceServiceTests: XCTestCase {
    func testMaterializeIsIdempotentAndClampsDay() throws {
        let db = try DatabaseClient.makeInMemory()
        let categories = CategoryService(repository: CategoryRepository(db: db))
        let service = RecurrenceService(repository: RecurrenceRepository(db: db), categories: categories)
        _ = try service.create(CreateRecurrenceDTO(
            kind: .expense, label: "Loyer", amountCents: 89_000, categoryId: "logement",
            dayOfMonth: 31, startYear: 2026, startMonth: 1, note: nil
        ))
        XCTAssertEqual(try service.materialize(year: 2026, month: 2), 1)
        XCTAssertEqual(try service.materialize(year: 2026, month: 2), 0)
        let items = try TransactionRepository(db: db).fetch(filter: FilterTransactionsDTO(year: 2026, month: 2))
        XCTAssertEqual(items.count, 1)
        XCTAssertEqual(items[0].transactionDate, "2026-02-28")
        XCTAssertEqual(items[0].status, .pending)
        XCTAssertNotNil(items[0].recurringRuleId)
    }

    func testDoesNotBackfillBeforeStart() throws {
        let db = try DatabaseClient.makeInMemory()
        let categories = CategoryService(repository: CategoryRepository(db: db))
        let service = RecurrenceService(repository: RecurrenceRepository(db: db), categories: categories)
        _ = try service.create(CreateRecurrenceDTO(
            kind: .income, label: "Salaire", amountCents: 200_000, categoryId: "salaire",
            dayOfMonth: 28, startYear: 2026, startMonth: 8, note: nil
        ))
        XCTAssertEqual(try service.materialize(year: 2026, month: 7), 0)
        XCTAssertEqual(try service.materialize(year: 2026, month: 8), 1)
    }
}
```

- [ ] **Step 2: Run — FAIL**

Run:

```bash
cd afterbudget-ios && xcodegen generate && xcodebuild test -scheme AfterBudget -destination 'platform=iOS Simulator,name=iPhone 16' -only-testing:AfterBudgetTests/RecurrenceServiceTests
```

Expected: FAIL

- [ ] **Step 3: Implement**

```swift
// afterbudget-ios/AfterBudget/Features/Recurrences/Models/RecurringRule.swift
import Foundation

struct RecurringRule: Identifiable, Equatable, Sendable {
    let id: String
    var kind: TransactionKind
    var label: String
    var amount: Money
    var categoryId: String
    var dayOfMonth: Int
    var startYear: Int
    var startMonth: Int
    var endYear: Int?
    var endMonth: Int?
    var note: String?
    var isActive: Bool
    var createdAt: String
    var updatedAt: String

    func covers(year: Int, month: Int) -> Bool {
        guard isActive, (1...12).contains(month) else { return false }
        let target = year * 12 + month - 1
        let start = startYear * 12 + startMonth - 1
        guard target >= start else { return false }
        if let endYear, let endMonth {
            return target <= endYear * 12 + endMonth - 1
        }
        return true
    }

    func dateString(year: Int, month: Int) -> String? {
        guard covers(year: year, month: month) else { return nil }
        let day = min(dayOfMonth, Self.daysInMonth(year: year, month: month))
        return String(format: "%04d-%02d-%02d", year, month, day)
    }

    static func daysInMonth(year: Int, month: Int) -> Int {
        var comps = DateComponents(); comps.year = year; comps.month = month; comps.day = 1
        let cal = Calendar(identifier: .gregorian)
        guard let date = cal.date(from: comps), let range = cal.range(of: .day, in: .month, for: date) else { return 28 }
        return range.count
    }
}
```

```swift
// afterbudget-ios/AfterBudget/Features/Recurrences/DTOs/CreateRecurrenceDTO.swift
import Foundation

struct CreateRecurrenceDTO: Equatable, Sendable {
    var kind: TransactionKind
    var label: String
    var amountCents: Int64
    var categoryId: String
    var dayOfMonth: Int
    var startYear: Int
    var startMonth: Int
    var note: String?
}
```

```swift
// afterbudget-ios/AfterBudget/Features/Recurrences/Repository/RecurrenceRepository.swift
import Foundation
import GRDB

struct RecurrenceRepository: Sendable {
    let db: DatabaseClient

    func insert(_ rule: RecurringRule) throws {
        try db.dbQueue.write { db in
            try db.execute(
                sql: """
                INSERT INTO recurring_rules
                (id, kind, label, amount_cents, category_id, day_of_month, start_year, start_month,
                 end_year, end_month, note, is_active, created_at, updated_at)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                """,
                arguments: [
                    rule.id, rule.kind.rawValue, rule.label, rule.amount.cents, rule.categoryId,
                    rule.dayOfMonth, rule.startYear, rule.startMonth, rule.endYear, rule.endMonth,
                    rule.note, rule.isActive ? 1 : 0, rule.createdAt, rule.updatedAt
                ]
            )
        }
    }

    func list() throws -> [RecurringRule] {
        try db.dbQueue.read { db in
            try Row.fetchAll(db, sql: "SELECT * FROM recurring_rules ORDER BY label").map(Self.map)
        }
    }

    func delete(id: String) throws {
        try db.dbQueue.write { db in
            try db.execute(sql: "DELETE FROM recurring_rules WHERE id = ?", arguments: [id])
        }
    }

    func occurrenceExists(ruleId: String, year: Int, month: Int) throws -> Bool {
        let prefix = String(format: "%04d-%02d", year, month)
        return try db.dbQueue.read { db in
            let n = try Int.fetchOne(db, sql: """
                SELECT COUNT(*) FROM transactions
                WHERE recurring_rule_id = ? AND substr(transaction_date,1,7) = ?
                """, arguments: [ruleId, prefix]) ?? 0
            return n > 0
        }
    }

    func insertOccurrence(rule: RecurringRule, date: String) throws {
        let now = ISO8601DateFormatter().string(from: Date())
        try db.dbQueue.write { db in
            try db.execute(
                sql: """
                INSERT INTO transactions
                (id, kind, label, amount_cents, transaction_date, status, category_id, note,
                 recurring_rule_id, created_at, updated_at)
                VALUES (?, ?, ?, ?, ?, 'pending', ?, ?, ?, ?, ?)
                """,
                arguments: [
                    UUID().uuidString.lowercased(), rule.kind.rawValue, rule.label,
                    rule.amount.cents, date, rule.categoryId, rule.note, rule.id, now, now
                ]
            )
        }
    }

    private static func map(_ row: Row) -> RecurringRule {
        RecurringRule(
            id: row["id"], kind: TransactionKind(rawValue: row["kind"]) ?? .expense,
            label: row["label"], amount: Money(cents: row["amount_cents"]), categoryId: row["category_id"],
            dayOfMonth: row["day_of_month"], startYear: row["start_year"], startMonth: row["start_month"],
            endYear: row["end_year"], endMonth: row["end_month"], note: row["note"],
            isActive: (row["is_active"] as Int) == 1, createdAt: row["created_at"], updatedAt: row["updated_at"]
        )
    }
}
```

```swift
// afterbudget-ios/AfterBudget/Features/Recurrences/Service/RecurrenceService.swift
import Foundation

struct RecurrenceService: RecurrenceMaterializing, Sendable {
    let repository: RecurrenceRepository
    let categories: CategoryService

    func create(_ dto: CreateRecurrenceDTO) throws -> RecurringRule {
        let label = try TransactionValidator.validateLabel(dto.label)
        let amount = try TransactionValidator.validateAmountCents(dto.amountCents)
        guard (1...31).contains(dto.dayOfMonth) else {
            throw AppError.validation("Le jour du mois doit être entre 1 et 31.")
        }
        guard (1...12).contains(dto.startMonth) else {
            throw AppError.validation("Mois de début invalide.")
        }
        _ = try categories.require(id: dto.categoryId, kind: dto.kind)
        let now = ISO8601DateFormatter().string(from: Date())
        let rule = RecurringRule(
            id: UUID().uuidString.lowercased(), kind: dto.kind, label: label, amount: amount,
            categoryId: dto.categoryId, dayOfMonth: dto.dayOfMonth, startYear: dto.startYear,
            startMonth: dto.startMonth, endYear: nil, endMonth: nil, note: dto.note,
            isActive: true, createdAt: now, updatedAt: now
        )
        try repository.insert(rule)
        return rule
    }

    func list() throws -> [RecurringRule] { try repository.list() }
    func delete(id: String) throws { try repository.delete(id: id) }

    @discardableResult
    func materialize(year: Int, month: Int) throws -> Int {
        var created = 0
        for rule in try repository.list() where rule.covers(year: year, month: month) {
            if try repository.occurrenceExists(ruleId: rule.id, year: year, month: month) { continue }
            guard let date = rule.dateString(year: year, month: month) else { continue }
            try repository.insertOccurrence(rule: rule, date: date)
            created += 1
        }
        return created
    }
}
```

- [ ] **Step 4: Run — PASS** puis brancher `BudgetService(recurrences: recurrenceService)` dans le container

Run:

```bash
cd afterbudget-ios && xcodegen generate && xcodebuild test -scheme AfterBudget -destination 'platform=iOS Simulator,name=iPhone 16' -only-testing:AfterBudgetTests/RecurrenceServiceTests
```

Expected: PASS

- [ ] **Step 5: Commit**

```bash
cd afterbudget-ios && git add AfterBudget/Features/Recurrences AfterBudgetTests/Recurrences && git commit -m "feat: add recurring rules with idempotent monthly materialization"
```

---

### Task 13: Settings UI + ImportExport backup/restore + tests

**Files:**
- Create: `afterbudget-ios/AfterBudget/Features/Settings/ViewModels/SettingsViewModel.swift`
- Create: `afterbudget-ios/AfterBudget/Features/Settings/Views/SettingsView.swift`
- Create: `afterbudget-ios/AfterBudget/Features/ImportExport/Service/ImportExportService.swift`
- Create: `afterbudget-ios/AfterBudget/Features/ImportExport/Views/ImportExportSection.swift`
- Create: `afterbudget-ios/AfterBudget/Features/Recurrences/Views/RecurrencesSettingsSection.swift`
- Test: `afterbudget-ios/AfterBudgetTests/ImportExport/ImportExportServiceTests.swift`

**Interfaces:**
- Consumes: `SettingsService`, `DatabaseClient`, `RecurrenceService`
- Produces: `ImportExportService.exportBackup(to:)` / `restoreBackup(from:)` ; UI Réglages

- [ ] **Step 1: Write failing backup test**

```swift
// afterbudget-ios/AfterBudgetTests/ImportExport/ImportExportServiceTests.swift
import XCTest
@testable import AfterBudget

final class ImportExportServiceTests: XCTestCase {
    func testExportAndRestorePreservesTransactions() throws {
        let url = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString + ".sqlite")
        defer { try? FileManager.default.removeItem(at: url) }
        let db = try DatabaseClient.makeOnDisk(url: url)
        let categories = CategoryService(repository: CategoryRepository(db: db))
        let tx = TransactionService(repository: TransactionRepository(db: db), categories: categories)
        _ = try tx.create(CreateTransactionDTO(
            kind: .income, label: "Salaire", amountCents: 1000,
            transactionDate: "2026-09-10", status: .completed, categoryId: "salaire", note: nil
        ))
        let exportURL = FileManager.default.temporaryDirectory.appendingPathComponent("backup.sqlite")
        defer { try? FileManager.default.removeItem(at: exportURL) }
        let service = ImportExportService(databaseURL: url, settings: SettingsService(repository: SettingsRepository(db: db)))
        try service.exportBackup(to: exportURL)
        try db.dbQueue.write { db in try db.execute(sql: "DELETE FROM transactions;") }
        XCTAssertEqual(try tx.list(FilterTransactionsDTO(year: 2026, month: 9)).count, 0)
        let restored = try service.restoreBackup(from: exportURL)
        let tx2 = TransactionService(
            repository: TransactionRepository(db: restored),
            categories: CategoryService(repository: CategoryRepository(db: restored))
        )
        XCTAssertEqual(try tx2.list(FilterTransactionsDTO(year: 2026, month: 9)).count, 1)
    }
}
```

- [ ] **Step 2: Run — FAIL**

Run:

```bash
cd afterbudget-ios && xcodegen generate && xcodebuild test -scheme AfterBudget -destination 'platform=iOS Simulator,name=iPhone 16' -only-testing:AfterBudgetTests/ImportExportServiceTests
```

Expected: FAIL

- [ ] **Step 3: Implement ImportExport + Settings UI**

```swift
// afterbudget-ios/AfterBudget/Features/ImportExport/Service/ImportExportService.swift
import Foundation
import GRDB

struct ImportExportService: Sendable {
    let databaseURL: URL
    let settings: SettingsService

    func exportBackup(to destination: URL) throws {
        let source = try DatabaseQueue(path: databaseURL.path)
        try source.barrierWriteWithoutTransaction { db in
            try db.execute(sql: "VACUUM INTO ?", arguments: [destination.path])
        }
        try settings.markExported(at: ISO8601DateFormatter().string(from: Date()))
    }

    func restoreBackup(from source: URL) throws -> DatabaseClient {
        let fm = FileManager.default
        if fm.fileExists(atPath: databaseURL.path) {
            try fm.removeItem(at: databaseURL)
        }
        try fm.copyItem(at: source, to: databaseURL)
        return try DatabaseClient.makeOnDisk(url: databaseURL)
    }
}
```

```swift
// afterbudget-ios/AfterBudget/Features/Settings/ViewModels/SettingsViewModel.swift
import Foundation

@MainActor
final class SettingsViewModel: ObservableObject {
    @Published var balanceInput: String = "0"
    @Published var overdraftInput: String = "0"
    @Published var theme: ThemeMode = .system
    @Published var message: String?
    @Published var rules: [RecurringRule] = []
    let settings: SettingsService
    let recurrences: RecurrenceService
    init(settings: SettingsService, recurrences: RecurrenceService) {
        self.settings = settings; self.recurrences = recurrences
    }
    func reload() {
        do {
            let s = try settings.fetch()
            balanceInput = Money(cents: s.currentBalance.cents).formatFR()
                .replacingOccurrences(of: " €", with: "")
            overdraftInput = Money(cents: s.overdraftLimit.cents).formatFR()
                .replacingOccurrences(of: " €", with: "")
            theme = s.theme
            rules = try recurrences.list()
            message = nil
        } catch { message = error.localizedDescription }
    }
    func save() {
        do {
            let balance = try Money.fromInput(balanceInput)
            let overdraft = try Money.fromInput(overdraftInput)
            try settings.update(UpdateSettingsDTO(
                currentBalanceCents: balance.cents,
                overdraftLimitCents: overdraft.cents,
                theme: theme
            ))
            message = "Enregistré."
        } catch { message = error.localizedDescription }
    }
    func reset() {
        do { try settings.resetAll(); reload(); message = "Données réinitialisées." }
        catch { message = error.localizedDescription }
    }
}
```

```swift
// afterbudget-ios/AfterBudget/Features/Settings/Views/SettingsView.swift
import SwiftUI

struct SettingsView: View {
    @ObservedObject var viewModel: SettingsViewModel
    var importExport: ImportExportSection
    var body: some View {
        NavigationStack {
            Form {
                Section("Compte") {
                    TextField("Solde actuel", text: $viewModel.balanceInput).keyboardType(.decimalPad)
                    TextField("Découvert autorisé", text: $viewModel.overdraftInput).keyboardType(.decimalPad)
                    Picker("Thème", selection: $viewModel.theme) {
                        ForEach(ThemeMode.allCases, id: \.self) { mode in
                            Text(mode.displayName).tag(mode)
                        }
                    }
                    Button("Enregistrer") { viewModel.save() }
                }
                RecurrencesSettingsSection(viewModel: viewModel)
                importExport
                Section {
                    Button("Réinitialiser les données", role: .destructive) { viewModel.reset() }
                }
                if let message = viewModel.message { Text(message) }
            }
            .navigationTitle("Réglages")
            .onAppear { viewModel.reload() }
        }
    }
}
```

```swift
// afterbudget-ios/AfterBudget/Features/Recurrences/Views/RecurrencesSettingsSection.swift
import SwiftUI

struct RecurrencesSettingsSection: View {
    @ObservedObject var viewModel: SettingsViewModel
    var body: some View {
        Section("Récurrences") {
            if viewModel.rules.isEmpty {
                Text("Aucune règle").foregroundStyle(.secondary)
            } else {
                ForEach(viewModel.rules) { rule in
                    HStack {
                        VStack(alignment: .leading) {
                            Text(rule.label)
                            Text("Le \(rule.dayOfMonth) de chaque mois").font(TypographyTokens.caption)
                        }
                        Spacer()
                        ABAmountText(money: rule.amount, kind: rule.kind)
                    }
                }
            }
        }
    }
}
```

```swift
// afterbudget-ios/AfterBudget/Features/ImportExport/Views/ImportExportSection.swift
import SwiftUI

struct ImportExportSection: View {
    let service: ImportExportService
    var onRestored: (DatabaseClient) -> Void
    @State private var message: String?
    @State private var showImporter = false
    var body: some View {
        Section("Sauvegarde") {
            Button("Exporter la base") {
                do {
                    let url = FileManager.default.temporaryDirectory
                        .appendingPathComponent("afterbudget-backup.sqlite")
                    try service.exportBackup(to: url)
                    message = "Export : \(url.lastPathComponent)"
                } catch { message = error.localizedDescription }
            }
            Button("Importer une base") { showImporter = true }
            if let message { Text(message).font(TypographyTokens.caption) }
        }
        .fileImporter(isPresented: $showImporter, allowedContentTypes: [.data]) { result in
            do {
                let url = try result.get()
                let db = try service.restoreBackup(from: url)
                onRestored(db)
                message = "Import terminé."
            } catch { message = error.localizedDescription }
        }
    }
}
```

- [ ] **Step 4: Run — PASS**

Run:

```bash
cd afterbudget-ios && xcodegen generate && xcodebuild test -scheme AfterBudget -destination 'platform=iOS Simulator,name=iPhone 16' -only-testing:AfterBudgetTests/ImportExportServiceTests
```

Expected: PASS

- [ ] **Step 5: Commit**

```bash
cd afterbudget-ios && git add AfterBudget/Features/Settings AfterBudget/Features/ImportExport AfterBudget/Features/Recurrences/Views AfterBudgetTests/ImportExport && git commit -m "feat: add settings UI and SQLite backup/restore"
```

---

### Task 14: App shell TabView + DI + theme

**Files:**
- Modify: `afterbudget-ios/AfterBudget/App/AfterBudgetApp.swift`
- Create: `afterbudget-ios/AfterBudget/App/AppDependencyContainer.swift`
- Modify: `afterbudget-ios/AfterBudget/App/RootView.swift`
- Test: `afterbudget-ios/AfterBudgetTests/App/RootViewModelTests.swift`

**Interfaces:**
- Consumes: tous les services
- Produces: `TabView` Accueil | Transactions | Stats | Réglages ; `fullScreenCover` onboarding ; theme from `AppSettings.theme`

- [ ] **Step 1: Write failing container test**

```swift
// afterbudget-ios/AfterBudgetTests/App/RootViewModelTests.swift
import XCTest
@testable import AfterBudget

@MainActor
final class RootViewModelTests: XCTestCase {
    func testShowsOnboardingWhenNotCompleted() throws {
        let container = try AppDependencyContainer.makeInMemory()
        let root = RootViewModel(container: container)
        XCTAssertTrue(root.showOnboarding)
        try container.settings.completeOnboarding(balance: .zero, overdraft: .zero)
        root.refreshSettings()
        XCTAssertFalse(root.showOnboarding)
    }
}
```

- [ ] **Step 2: Run — FAIL**

Run:

```bash
cd afterbudget-ios && xcodegen generate && xcodebuild test -scheme AfterBudget -destination 'platform=iOS Simulator,name=iPhone 16' -only-testing:AfterBudgetTests/RootViewModelTests
```

Expected: FAIL

- [ ] **Step 3: Implement DI + shell**

```swift
// afterbudget-ios/AfterBudget/App/AppDependencyContainer.swift
import Foundation

@MainActor
final class AppDependencyContainer: ObservableObject {
    let db: DatabaseClient
    let categories: CategoryService
    let settings: SettingsService
    let transactions: TransactionService
    let recurrences: RecurrenceService
    let budget: BudgetService
    let statistics: StatisticsService
    let importExport: ImportExportService

    init(db: DatabaseClient, databaseURL: URL) {
        self.db = db
        let categoryRepo = CategoryRepository(db: db)
        let categories = CategoryService(repository: categoryRepo)
        let settings = SettingsService(repository: SettingsRepository(db: db))
        let transactions = TransactionService(repository: TransactionRepository(db: db), categories: categories)
        let recurrences = RecurrenceService(repository: RecurrenceRepository(db: db), categories: categories)
        self.categories = categories
        self.settings = settings
        self.transactions = transactions
        self.recurrences = recurrences
        self.budget = BudgetService(settings: settings, transactions: transactions, recurrences: recurrences)
        self.statistics = StatisticsService(transactions: TransactionRepository(db: db), categories: categoryRepo)
        self.importExport = ImportExportService(databaseURL: databaseURL, settings: settings)
    }

    static func makeInMemory() throws -> AppDependencyContainer {
        let url = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString + ".sqlite")
        return AppDependencyContainer(db: try DatabaseClient.makeOnDisk(url: url), databaseURL: url)
    }

    static func makeDefault() throws -> AppDependencyContainer {
        let url = try DatabaseClient.defaultDatabaseURL()
        return AppDependencyContainer(db: try DatabaseClient.makeOnDisk(url: url), databaseURL: url)
    }
}
```

```swift
// afterbudget-ios/AfterBudget/App/RootView.swift
import SwiftUI

@MainActor
final class RootViewModel: ObservableObject {
    @Published var showOnboarding: Bool = true
    @Published var theme: ThemeMode = .system
    let container: AppDependencyContainer
    init(container: AppDependencyContainer) {
        self.container = container
        refreshSettings()
    }
    func refreshSettings() {
        do {
            let s = try container.settings.fetch()
            showOnboarding = !s.onboardingCompleted
            theme = s.theme
        } catch { showOnboarding = true }
    }
}

struct RootView: View {
    @StateObject private var root: RootViewModel
    init(container: AppDependencyContainer) {
        _root = StateObject(wrappedValue: RootViewModel(container: container))
    }
    var body: some View {
        let cal = Calendar.current
        let year = cal.component(.year, from: Date())
        let month = cal.component(.month, from: Date())
        TabView {
            BudgetDashboardView(viewModel: BudgetDashboardViewModel(budget: root.container.budget, year: year, month: month))
                .tabItem { Label("Accueil", systemImage: "house") }
            TransactionListView(viewModel: TransactionListViewModel(service: root.container.transactions, year: year, month: month))
                .tabItem { Label("Transactions", systemImage: "list.bullet") }
            StatisticsView(viewModel: StatisticsViewModel(service: root.container.statistics, year: year, month: month))
                .tabItem { Label("Stats", systemImage: "chart.bar") }
            SettingsView(
                viewModel: SettingsViewModel(settings: root.container.settings, recurrences: root.container.recurrences),
                importExport: ImportExportSection(service: root.container.importExport, onRestored: { _ in root.refreshSettings() })
            )
            .tabItem { Label("Réglages", systemImage: "gearshape") }
        }
        .tint(ColorTokens.hex(ColorTokens.Light.accentHex))
        .preferredColorScheme(colorScheme)
        .fullScreenCover(isPresented: $root.showOnboarding) {
            OnboardingView(viewModel: OnboardingViewModel(settings: root.container.settings), onFinished: { root.refreshSettings() })
        }
    }
    private var colorScheme: ColorScheme? {
        switch root.theme {
        case .system: return nil
        case .light: return .light
        case .dark: return .dark
        }
    }
}
```

```swift
// afterbudget-ios/AfterBudget/App/AfterBudgetApp.swift
import SwiftUI

@main
struct AfterBudgetApp: App {
    @StateObject private var container = try! AppDependencyContainer.makeDefault()
    var body: some Scene {
        WindowGroup {
            RootView(container: container)
                .environmentObject(container)
        }
    }
}
```

- [ ] **Step 4: Run — PASS**

Run:

```bash
cd afterbudget-ios && xcodegen generate && xcodebuild test -scheme AfterBudget -destination 'platform=iOS Simulator,name=iPhone 16' -only-testing:AfterBudgetTests/RootViewModelTests
```

Expected: PASS

- [ ] **Step 5: Commit**

```bash
cd afterbudget-ios && git add AfterBudget/App AfterBudgetTests/App && git commit -m "feat: wire TabView shell, DI container and onboarding cover"
```

---

### Task 15: docs FR — ARCHITECTURE, SPEC, DATABASE, DESIGN_SYSTEM, DECISIONS

**Files:**
- Create: `afterbudget-ios/docs/ARCHITECTURE.md`
- Create: `afterbudget-ios/docs/SPEC.md`
- Create: `afterbudget-ios/docs/DATABASE.md`
- Create: `afterbudget-ios/docs/DESIGN_SYSTEM.md`
- Create: `afterbudget-ios/docs/DECISIONS.md`

**Interfaces:**
- Consumes: décisions verrouillées + implémentation tasks 1–14
- Produces: documentation française alignée code (pas Encre & Cuivre)

- [ ] **Step 1: Write ARCHITECTURE.md**

~~~~markdown
# ARCHITECTURE.md — AfterBudget iOS

## Organisation

Application SwiftUI iPhone-only (iOS 17+), MVVM par feature, persistence GRDB/SQLite offline.

    AfterBudget/
      App/                 — entrypoint, DI, RootView (TabView)
      Core/Domain/         — Money, kinds, FinancialStatus, AppError
      Core/Database/       — DatabaseClient, migrations, seeds
      Core/DesignSystem/   — tokens charcoal/paper/matte copper + composants
      Features/<Name>/     — Models, DTOs, Validators, Repository, Service, ViewModels, Views

## Circulation d’une action

    View → ViewModel → Service → Validator/DTO → Repository → GRDB/SQLite → reload ViewModel

## Règles

- Aucun SQL dans les Views / ViewModels
- Montants exclusivement `Int64` centimes via `Money`
- Pas de module réseau / mise à jour distante
- 1 fichier logique ↔ 1 fichier de tests
~~~~

- [ ] **Step 2: Write SPEC.md**

~~~~markdown
# SPEC.md — AfterBudget iOS (extrait aligné desktop)

## Solde prévisionnel

    projected = currentBalance + pendingIncome - pendingExpenses

## Statuts

- Healthy : `projected >= 0` → « En forme »
- Attention : `projected < 0 && projected >= -overdraft`
- Danger : `projected < -overdraft`

## Exemples

| Cas | Solde | Pending income | Pending expense | Découvert | Projected | Statut |
|---|---:|---:|---:|---:|---:|---|
| 22.1 | -360 | 1207 | 1325 | 500 | -478 | Attention |
| 22.2 | 300 | 1500 | 1200 | 200 | 600 | En forme |
| 22.3 | -400 | 200 | 500 | 500 | -700 | Danger |

Montants stockés en centimes (`-478,00 €` → `-47800`).
Les transactions `completed` sont déjà dans le solde courant : seuls les `pending` du mois actif entrent dans la prévision.
~~~~

- [ ] **Step 3: Write DATABASE.md**

```markdown
# DATABASE.md — Schéma SQLite iOS

Tables alignées desktop : `categories`, `transactions`, `app_settings`, `recurring_rules`, `schema_migrations`.

## transactions
id TEXT PK, kind, label, amount_cents (>0), transaction_date, status, category_id FK,
recurring_rule_id NULL FK, note, created_at, updated_at

## recurring_rules
id TEXT PK, kind, label, amount_cents, category_id, day_of_month 1..31,
start_year/start_month, end_year/end_month NULL, note, is_active, created_at, updated_at

## categories
id TEXT PK, kind, name, icon, color #RRGGBB, sort_order, is_default, is_active, created_at, updated_at

## app_settings
id=1, current_balance_cents, overdraft_limit_cents (>=0), currency_code, locale, theme,
balance_updated_at, onboarding_completed, last_export_date NULL, created_at, updated_at

## Migrations iOS
- v1 initial + 35 catégories + settings row
- v2 recurring_rules + transactions.recurring_rule_id
- v3 last_export_date
Pas de v4 `ignored_update_version` (pas de module réseau).
```

- [ ] **Step 4: Write DESIGN_SYSTEM.md**

```markdown
# DESIGN_SYSTEM.md — Charcoal / Paper / Matte Copper

Direction éditoriale sobre pour iOS. **Ce n’est pas Encre & Cuivre** (desktop).

## Light
- paper `#F7F5F1`, charcoal `#1C1B19`, matte copper `#8B6F47`
- surface `#FFFDF9`, muted `#6F6A62`
- income `#2F6B4F`, expense `#9B3A3A`
- healthy `#2F6B4F`, attention `#8A6A1F`, danger `#9B3A3A`

## Dark
- charcoal `#121110`, accent `#C4A574`, surface `#1C1A18`

## Composants
- `ABAmountText` — montants mono rounded, préfixe +/− selon kind
- `ABStatusPill` — pastille En forme / Attention / Danger
```

- [ ] **Step 5: Write DECISIONS.md**

```markdown
# DECISIONS.md — AfterBudget iOS

## GRDB plutôt que Core Data / SwiftData
Alignement SQLite desktop, migrations SQL explicites, tests in-memory simples.

## XcodeGen
`project.yml` versionné ; `.xcodeproj` généré, non commité.

## FinancialStatus.attention (pas warning)
Alignement wording produit FR « Attention » et décisions verrouillées iOS.

## Pas de module de mise à jour réseau
Offline strict ; schéma sans `ignored_update_version`.

## Design distinct du desktop
Palette éditoriale charcoal/paper/matte copper pour iOS ; desktop conserve Encre & Cuivre.
```

- [ ] **Step 6: Commit docs**

```bash
cd afterbudget-ios && git add docs && git commit -m "docs: add FR architecture, spec, database, design system, decisions"
```

- [ ] **Step 7: Full test suite**

Run:

```bash
cd afterbudget-ios && xcodegen generate && xcodebuild test -scheme AfterBudget -destination 'platform=iOS Simulator,name=iPhone 16'
```

Expected: `** TEST SUCCEEDED **`

---

## Self-Review

### 1. Spec coverage
| Exigence | Task |
|---|---|
| Scaffold XcodeGen + GRDB | 1 |
| Domain Money/kinds/status/theme/errors/FinancialStatus | 2 |
| DB + migrations + seeds | 3 |
| Design charcoal/paper/matte copper | 4 |
| Categories | 5 |
| Settings | 6 |
| BudgetCalculator SPEC −478/600/−700 | 7 |
| Transactions CRUD + list VM | 8 |
| Budget dashboard | 9 |
| Onboarding | 10 |
| Statistics | 11 |
| Recurrences materialize | 12 |
| Settings UI + ImportExport | 13 |
| TabView shell + DI + theme | 14 |
| Docs FR | 15 |
| Offline / no network update | Global + Task 15 DECISIONS |
| Amounts Int64 cents | Task 2 |
| Tab labels Accueil\|Transactions\|Stats\|Réglages | Task 14 |
| Bundle id / scheme | Task 1 |

### 2. Placeholder scan
Aucun TBD/TODO/« similar to ». Chaque step TDD contient du Swift réel et une commande `xcodebuild test` exacte.

### 3. Type consistency
Noms stables cross-tasks : `Money`, `TransactionKind`, `TransactionStatus`, `ThemeMode`, `AppError`, `FinancialStatus` (`.healthy|.attention|.danger`), `BudgetSummary`, `BudgetCalculator`, `MonthSums`, `Category`, `AppSettings`, `UpdateSettingsDTO`, `Transaction`, `CreateTransactionDTO`, `UpdateTransactionDTO`, `FilterTransactionsDTO`, `MonthlyStatistics`, `CategoryStats`, `RecurringRule`, `CreateRecurrenceDTO`, `RecurrenceMaterializing`, `DatabaseClient`, `AppDependencyContainer`.

### Gaps (honnêtes, hors blocage)
- Formulaire SwiftUI dédié « Nouvelle / Modifier transaction » : DTOs + service prêts (Task 8) ; UI liste présente — l’exécuteur peut ajouter une sheet sur `TransactionListView` sans changer les contrats.
- Création UI de récurrences : service + listing Réglages prêts ; sheet de création optionnelle.
- Pas de sync inter-appareils (hors scope offline).
