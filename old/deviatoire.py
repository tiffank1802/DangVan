import numpy as np
from math import *
import old.versDV as dv
# import versDV as dv

# def deviateur(tens):
    
#     p = dv.hydro(tens)
#     res = []
#     for i in range(3):
#         res.append(tens[i]-p)
#     for i in range(3,6):
#         res.append(tens[i])
#     return np.array(res)


def deviateur(tens):
    H=dv.hydro(tens)
    deviat=dv.tens_to_mat(tens)-np.eye(3)*H
    deviat=dv.mat_to_tens(deviat)
    deviat=np.delete(deviat, 2)  # on enlève la troisième
    return deviat

# def hydro(tens):
#     p = 0
#     for i in range(3):
#         p = p + tens[i]
#     return p/3

# def genereTens(sigma1,omega,pasTemps,fin):
#     tens = np.array([sigma1,0,0,0,0,0])
#     for i in range(int(fin/pasTemps)):
#         t = (i+1)*pasTemps
#         ligne = np.array([sigma1*cos(omega*t),0,0,0,0,0])
#         tens = np.vstack((tens, ligne))
#     # omega est la pulsation, vous pouvez choisir 2*pi par exemple
#     # sigma1 est fixe, par exemple 100 MPa
#     # cette fonction doit générer une matrice de 6 colonnes, chaque ligne étant le tenseur à un instant du cycle, et de la forme [sigma1*cos(omega*t),0,0,0,0,0]
#     return tens

# def CalculMatDev(matTens):
#     resDev = matTens[:,[0,1,3,4,5]]
#     taille = resDev.shape
#     nLig = taille[0]
#     for i in range(nLig):
#         pH = hydro(matTens[i])
#         for j in range(3):
#             resDev[i][j] = resDev[i][j]-pH
#     return resDev
def CalculMatDev(matTens):
    return np.array([deviateur(matTens[i]) for i in range(matTens.shape[0])])   
def normeTresca(tens):
    TensM=dv.tens_to_mat(tens)
    valP = np.linalg.eigvals(TensM)
    max = float(np.max(valP))
    min = float(np.min(valP))
    return abs(max-min)

def diametre(matTens):
    """calcul la distance maximale entre deux lignes (des déviateurs) au sens de la norme de Tresca) et retourne les deux points extrêmes, """
    
    matDev=CalculMatDev(matTens)
    point1=np.zeros(5)
    point2=np.zeros(5)
    maxDist=0
    for i in range(matDev.shape[0]-1):
        k=i
        point1 = matDev[k]
        for j in range(i+1,matDev.shape[0]):
            point2 = matDev[j]
            dist=normeTresca(matDev[i]-matDev[j])
            if dist>maxDist:
                    maxDist=dist
                    point1=matDev[k]
                    point2=matDev[j]
                    
    return maxDist, point1, point2
    

def recentre(matTens):
    """retourne une matrice de tenseurs déviateurs recentrée par Centre qui est un tenseur."""
#    taille = matDev.shape
#   nLig = taille[0]
#    for i in range(nLig):
#        ligne = matDev[i] - Centre 
#        matCentre = np.vstack((matCentre, ligne))
    points = diametre(matTens)
    Centre = (points[1] + points[2])/2
    matDev=CalculMatDev(matTens)
    return [matDev - Centre, Centre]

# class Deviateur:


if __name__ == "__main__":
    matTens = dv.load_tens_from_csv('./datas/tensors_uniaxial.csv')
    res = CalculMatDev(matTens)

# trace le nuage de points J2(matDev)(t),p(t))