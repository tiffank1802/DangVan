import old.versDV as dv
import matplotlib.pyplot as plt

# Generate points for uniaxial loading
points1 = dv.nuage(100, 2*dv.pi, 0.01, 1)

# Plot and save uniaxial
plt.figure()
plt.scatter(points1[:, 0], points1[:, 1])
plt.xlabel("pression hydrostatique")
plt.ylabel("amplitude de cisaillement max")
plt.title("Nuage de points - Chargement Uniaxial")
plt.savefig('/root/DangVan/PRI_Dangvan/images/uniaxial.png')
plt.close()

# Generate points for torsion
points2 = dv.nuageOrt(100, 2*dv.pi, 0.01, 1)

# Plot and save torsion
plt.figure()
plt.scatter(points2[:, 0], points2[:, 1])
plt.xlabel("pression hydrostatique")
plt.ylabel("amplitude de cisaillement max")
plt.title("Nuage de points - Torsion Pure")
plt.savefig('/root/DangVan/PRI_Dangvan/images/torsion.png')
plt.close()