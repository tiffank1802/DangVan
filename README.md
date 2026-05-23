# Critère de Dang Van — Analyse de contraintes et plans critiques

> Projet numérique d'analyse de fatigue multiaxiale selon le critère de Dang Van.  
> Implémentation en Python et comparaison de deux méthodes : l'approche originelle de 1973 et une version allégée inspirée de 1989.

**Auteurs :** Paul Lorthioir, Kevin Tongue  
**Encadrants :** Éric Feulvarch, Françoise Fauvin  

---

## Table des matières

- [Contexte](#contexte)
- [Critère de Dang Van](#critère-de-dang-van)
- [Méthodes implémentées](#méthodes-implémentées)
- [Structure du projet](#structure-du-projet)
- [Résultats](#résultats)
- [Paramètres matériaux](#paramètres-matériaux-a-et-b)
- [Références](#références)

---

## Contexte

Le dimensionnement des structures soumises à des chargements cycliques complexes est un enjeu critique en ingénierie mécanique. Ce projet implémente numériquement le **critère de Dang Van**, une référence dans l'analyse de la fatigue multiaxiale, et compare deux formulations algorithmiques sur des cas de référence (traction, torsion) et des cas multiaxiaux complexes.

---

## Critère de Dang Van

Le critère repose sur une approche **à deux échelles** :

- **Échelle macroscopique** : les contraintes $\Sigma_{ij}(t)$ sont homogènes dans un volume élémentaire représentatif.
- **Échelle microscopique** : les contraintes locales intègrent les contraintes résiduelles stationnaires $\rho^*(y)$ à l'état d'adaptation (*shakedown*) :

$$\sigma(y,t) = \Sigma_{ij}(t) + \rho^*(y)$$

Le **critère de non-rupture** s'exprime :

$$\tau(t) + \alpha \cdot P(t) \leq \beta$$

où :
- $\tau(t)$ est le cisaillement instantané sur le plan critique,
- $P(t) = \text{tr}(\underline{\underline{\sigma}}(t))/3$ est la pression hydrostatique,
- $\alpha, \beta$ sont des constantes matériaux identifiées expérimentalement.

---

## Méthodes implémentées

### 1. Méthode originelle — Discrétisation angulaire (1973)

**Fichier :** `versDV.py`

Traduit littéralement la définition physique du critère par un balayage complet de l'espace des plans de glissement potentiels.

**Algorithme :**

1. Calcul de la pression hydrostatique $P(t) = (\sigma_{xx} + \sigma_{yy} + \sigma_{zz})/3$ à chaque pas de temps.
2. Balayage de toutes les normales $\vec{n}(\theta, \phi)$ avec $\theta, \phi \in [0°, 180°]$ par pas de $1°$.
3. Pour chaque facette, calcul du vecteur cisaillement :

$$\vec{\tau}(t) = \left(\underline{\underline{\Sigma}}(t) \cdot \vec{n}\right) - \left(\vec{n} \cdot \underline{\underline{\Sigma}}(t) \cdot \vec{n}\right)\vec{n}$$

4. Identification du **plan critique** maximisant l'amplitude de cisaillement sur le cycle.
5. Construction du nuage de points $(P(t), \tau(t))$ dans le diagramme de Dang Van.

> ⚠️ Méthode précise mais coûteuse en temps de calcul (~160× plus lente que la méthode allégée).

---

### 2. Méthode allégée — Espace des déviateurs (1989)

**Fichier :** `deviatoire.py`

Contourne le balayage angulaire en transposant le problème dans un **espace vectoriel à 5 dimensions** (espace des déviateurs).

**Algorithme :**

1. Décomposition du tenseur des contraintes :

$$\underline{\underline{\Sigma}}(t) = P(t) \cdot \underline{\underline{I}} + \underline{\underline{S}}(t)$$

2. Recherche du **milieu de la corde la plus éloignée** dans le nuage déviatorique pour estimer le centre $\underline{\underline{\Sigma}}^*$ (approximation du centre de la sphère circonscrite).

3. Calcul du déviateur recentré :

$$\underline{\underline{S}}_{\text{new}}(t) = \underline{\underline{S}}(t) - \underline{\underline{\Sigma}}^*$$

4. Cisaillement instantané via la norme de Tresca :

$$\tau(t) = \text{Tresca}\left(\underline{\underline{S}}_{\text{new}}(t)\right)$$

5. Vérification du critère et tracé du diagramme $(P(t), \tau(t))$.

> ✅ Environ **160 fois plus rapide** que la méthode originelle, avec des résultats identiques sur les cas de référence.

---

### 3. Méthode de référence — Logiciel ChronoCad (1989)

Pour la formulation exacte de la plus petite sphère circonscrite dans l'espace des déviateurs, un logiciel dédié (**ChronoCad**, basé sur les travaux *Fast Estimation*) est utilisé comme référence de comparaison.

---

## Structure du projet

```
DangVan/
├── versDV.py              # Méthode originelle 1973 (discrétisation angulaire)
├── deviatoire.py          # Méthode allégée 1989 (espace des déviateurs)
├── images/                # Figures et diagrammes de Dang Van
│   ├── traction-compression_methode_originelle.png
│   ├── traction_compression_methode_allégée.png
│   ├── torsion.png
│   ├── torsion_methode-allégée.png
│   ├── Multiaxial1.png
│   ├── Multiaxial2.png
│   └── dangvan_limite.png
└── README.md
```

---

## Résultats

### Traction-Compression uniaxiale

Tenseur de chargement sinusoïdal d'amplitude $\sigma_a = 100$ MPa :

$$\sigma(t) = \begin{pmatrix} \sigma_a \cos(\omega t) & 0 & 0 \\ 0 & 0 & 0 \\ 0 & 0 & 0 \end{pmatrix}$$

- Pression hydrostatique : $P(t) = \frac{\sigma_a}{3}\cos(\omega t)$
- Cisaillement sur le plan à 45° : $\tau_a = \frac{\sigma_a}{2} = 50$ MPa
- Trajet dans le diagramme : **droite oblique**
- Les trois méthodes donnent des **résultats identiques**.

---

### Torsion pure

$$\sigma(t) = \begin{pmatrix} 0 & \tau_a \cos(\omega t) & 0 \\ \tau_a \cos(\omega t) & 0 & 0 \\ 0 & 0 & 0 \end{pmatrix}$$

- Pression hydrostatique : $P(t) = 0$
- Cisaillement maximal : $\tau_a$
- Trajet dans le diagramme : **segment vertical sur l'axe des ordonnées**
- Les trois méthodes donnent des **résultats identiques**.

---

### Chargements multiaxiaux

**Cas 1 :**
$$\sigma(t) = \begin{pmatrix} \sigma_1\cos(\omega t) & -0.1\sigma_1\sin(\omega t) & 0 \\ -0.1\sigma_1\sin(\omega t) & 0 & -0.1\sigma_1\sin(2\omega t) \\ 0 & -0.1\sigma_1\sin(2\omega t) & 0 \end{pmatrix}, \quad \sigma_1 = 100 \text{ MPa}$$

**Cas 2 :**
$$\sigma(t) = \begin{pmatrix} 100\cos(2\pi t) & -50\cos(4\pi t) & 0 \\ -50\cos(4\pi t) & 0 & 30\cos(3\pi t) \\ 0 & 30\cos(3\pi t) & 0 \end{pmatrix}$$

Dans les deux cas, les résultats de l'algorithme sont **en accord avec ChronoCad**, validant l'implémentation.

---

## Paramètres matériaux a et b

Identifiés à partir de deux essais à la limite de rupture (à $t = 0$) :

| Essai | Équation |
|---|---|
| Traction-compression | $\frac{\sigma_1}{2} + a\frac{\sigma_1}{3} = b$ |
| Torsion pure | $\tau_1 + a \times 0 = b$, avec $\tau_1 = \frac{\sigma_1}{\sqrt{3}}$ |

**Résolution :**

$$b = \tau_1 = 50 \text{ MPa}$$
$$a = \sqrt{3} - \frac{3}{2} \approx 0.23$$

> **Observation :** Le chargement en traction dépasse le critère (rupture prédite), tandis que la torsion pure reste en-deçà de la limite de fatigue.

---

## Références

- Dang Van, K. (1973). *Sur la résistance à la fatigue des métaux.* Sciences et Techniques de l'Armement.
- Dang Van, K. (1989). *Macro-micro approach in high-cycle multiaxial fatigue.*
- Ballard, P. et al. *Fast Estimation of the Fatigue Limit by Dang Van Method.*