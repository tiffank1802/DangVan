# Plan de portage des algorithmes de Dang Van en Rust

## Objectif et avancement

Le socle Rust décrit ci-dessous est maintenant amorcé : types tensoriels, deux méthodes de calcul, import CSV, CLI et tests unitaires sont présents. Il reste à exécuter ces tests dans un environnement Rust, valider quantitativement tous les CSV et compléter les comparaisons de performance.

Porter les deux approches présentes dans le projet vers une bibliothèque Rust testable et réutilisable, sans dépendance à Python/NumPy :

1. **Méthode originelle (1973)** : recherche des plans critiques par discrétisation angulaire.
2. **Méthode déviatorique (1989, approximation actuelle)** : recentrage du nuage des déviateurs, puis calcul du cisaillement équivalent.

Le portage doit conserver les conventions des données existantes et rendre explicites les différences entre l'algorithme approximatif actuel et une recherche exacte de sphère circonscrite.

## État des lieux et précautions

- Les tenseurs CSV utilisent l'ordre `[xx, yy, zz, xy, xz, yz]`, avec cisaillements tensoriels (pas de facteur 2).
- La matrice symétrique correspondante est `[[xx, xy, xz], [xy, yy, yz], [xz, yz, zz]]`.
- La pression hydrostatique est `p = trace(σ) / 3`; le déviateur est `s = σ - p I`.
- Les implémentations Python ne sont pas parfaitement cohérentes : le code appelé « amplitude » maximise le cisaillement instantané, alors que le critère de fatigue demande de suivre **le même plan pendant tout le cycle** puis d'évaluer l'amplitude de sa trajectoire de cisaillement. Le portage devra implémenter cette définition cyclique et la tester, plutôt que reproduire cette confusion.
- La routine déviatorique actuelle cherche le diamètre par paires, mais calcule ses distances avec la norme J2 alors que sa documentation annonce Tresca. Le recentrage par le milieu de la paire la plus éloignée est une approximation, pas en général le centre exact de la plus petite sphère englobante. Ces choix doivent être distingués et documentés dans l'API.
- Les CSV versionnés fournissent des cas de référence reproductibles; les fonctions Python de génération et l'application Streamlit ne doivent pas être nécessaires au cœur Rust.

## Conventions mathématiques à figer

- Représenter le tenseur par ses six composantes `[xx, yy, zz, xy, xz, yz]`; utiliser un type dédié plutôt qu'un vecteur de longueur variable.
- Stocker les plans par une normale unitaire. Une normale `n` et `-n` représentent le même plan.
- Calculer le vecteur de cisaillement sur le plan : `τ⃗ = σ n - (n · σ n) n`.
- Pour un plan fixe, construire le centre de la trajectoire des vecteurs `τ⃗(t)` sur le cycle et définir l'amplitude comme le rayon maximal `max_t ||τ⃗(t) - centre||`. Maximiser ensuite cette amplitude sur les normales explorées. Conserver en résultat le plan critique et le centre associé.
- Pour la formulation déviatorique existante, exposer explicitement la métrique : `Tresca = λmax - λmin` selon la convention du code, ou `Tresca/2` si une amplitude de cisaillement physique est attendue. Ne pas mélanger cette grandeur avec la norme J2.
- Les unités sont celles d'entrée (les jeux actuels sont en MPa); aucune conversion implicite.

## Architecture proposée

```text
Cargo.toml
src/
  lib.rs             # API publique et erreurs
  tensor.rs          # Tensor3, conversion, hydrostatique, déviateur, valeurs propres
  loading.rs         # historique temporel de contraintes et validation
  angular.rs         # discrétisation des normales et calcul cyclique 1973
  deviatoric.rs      # diamètres, recentrage et critère approximatif 1989
  csv_io.rs          # lecture facultative des CSV du dépôt
src/bin/dangvan.rs   # CLI facultative pour analyser un CSV
 tests/
  tensor.rs
  angular.rs
  deviatoric.rs
  reference_cases.rs
```

Types/API indicatifs :

- `StressTensor { xx, yy, zz, xy, xz, yz }`
- `LoadingHistory { samples: Vec<StressTensor>, times: Option<Vec<f64>> }`
- `AngularConfig { step_degrees: f64 }`
- `CriterionResult { pressure: Vec<f64>, shear: Vec<f64>, critical_normal: [f64; 3], max_dang_van: f64 }`
- `DeviatoricConfig { metric: Metric, centering: Centering }`, où `Centering` différencie le milieu du diamètre (heuristique existante) d'une méthode exacte future.

Les fonctions valident les entrées et renvoient `Result<_, Error>` : historique vide, composante non finie, temps mal formés, pas angulaire invalide ou erreur de lecture CSV ne doivent pas provoquer de panique.

## Étapes de réalisation

### 1. Figer les conventions et les tests

- Ajouter un `Cargo.toml` pour une bibliothèque Rust (édition stable), en gardant le dépôt Python intact durant la transition.
- Écrire les tests de conversion matrice/vecteur, trace, pression hydrostatique, déviateur (trace nulle) et calcul du vecteur tangent.
- Valider les conventions sur les colonnes CSV avant toute optimisation.

### 2. Construire le socle tensoriel

- Implémenter les opérations sur tenseurs symétriques 3×3 avec calculs scalaires simples ou une petite bibliothèque linéaire si elle simplifie les valeurs propres.
- Implémenter une routine robuste de valeurs propres réelles pour tenseur symétrique (ou utiliser une dépendance Rust fiable) afin d'obtenir les critères Tresca/J2.
- Tester les invariants : invariance par rotation, hydrostatique pure sans déviateur et invariance de la trace sous extraction du déviateur.

### 3. Porter la méthode angulaire

- Générer les normales sphériques à partir d'un pas configurable; éviter les doublons aux pôles et gérer l'équivalence `n ~ -n`.
- Pour chaque normale, balayer les échantillons temporels, calculer `p(t)` et `τ⃗(t)`, puis l'amplitude cyclique du plan.
- Garder le maximum et sa normale; produire aussi les points `(p(t), amplitude)` utiles au diagramme.
- Prévoir un résultat déterministe et mesurer le coût en fonction du nombre de pas temporels et angulaires.

### 4. Porter la méthode déviatorique

- Convertir chaque historique en déviateurs dans un espace de dimension cinq (le tenseur 3×3 symétrique sans trace a cinq degrés de liberté).
- Implémenter la recherche par paires du diamètre dans la métrique sélectionnée, sans matrice carrée complète si elle n'est pas nécessaire.
- Recentrer autour du milieu de la paire extrême pour reproduire l'heuristique actuelle.
- Garder une interface permettant ultérieurement une vraie plus petite sphère englobante (algorithme exact adapté à cinq dimensions) et comparer les deux résultats.
- Signaler clairement qu'une méthode accélérée ou approximative n'est pas une référence exacte sans validation dédiée.

### 5. Import CSV et commandes

- Lire les colonnes nommées du dépôt et rejeter les lignes mal formées plutôt que supposer silencieusement un format.
- Ajouter éventuellement `dangvan analyse datas/tensors_uniaxial.csv --method angular --step-deg 1` et une sortie texte/CSV des résultats. Le tracé reste hors du cœur de calcul.

### 6. Validation, performance et migration

- Comparer Rust et Python sur les jeux existants, après harmonisation de la définition de l'amplitude et de la métrique.
- Mesurer les erreurs numériques et le temps d'exécution sur plusieurs pas angulaires; paralléliser seulement après obtention d'une version séquentielle de référence.
- Ajouter CI locale/automatisée avec `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` et `cargo test`.
- Une fois la parité validée, documenter les commandes Rust dans le README; garder ou retirer l'interface Python dans une étape séparée.

## Tests d'acceptation

1. **Hydrostatique** : `σ = q I` donne `p = q`, déviateur nul et cisaillement nul pour toute normale.
2. **Traction-compression uniaxiale** : pour une amplitude alternée `σa`, le plan à 45° donne une amplitude de cisaillement `σa / 2`; le résultat angulaire converge vers cette valeur quand le pas est raffiné.
3. **Torsion pure** : pour un cisaillement alterné `τa`, le cisaillement cyclique maximal doit être cohérent avec `τa` selon la convention d'amplitude retenue.
4. **Rotation** : une rotation orthogonale du chargement ne change ni les pressions ni le critère maximal; la normale critique tourne avec le tenseur.
5. **Régression CSV** : charger les trois fichiers `datas/tensors_uniaxial.csv`, `datas/tensors_torsion.csv` et `datas/tensor_multiaxiale.csv`; vérifier absence de NaN/infini, résultats déterministes, et écarts documentés avec Python lorsque ses conventions diffèrent.
6. **Limites** : historique vide, valeurs non finies, pas nul/négatif et CSV invalide retournent des erreurs explicites.

## Livrables prévus

- Une bibliothèque Rust documentée et testée, sans changement destructif des scripts Python.
- Les deux méthodes disponibles derrière une API commune avec leurs hypothèses explicites.
- Un rapport de validation des cas analytiques et des CSV, incluant les écarts de convention identifiés.
