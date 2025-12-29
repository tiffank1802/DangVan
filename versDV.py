# Cellule 1 : Installation des packages (optionnel)
# !pip install numpy matplotlib

# Cellule 2 : Import des bibliothèques
import numpy as np
import matplotlib.pyplot as plt
from math import *
import cmath
import scipy.special as sp

# Cellule 3 : Fonction normale
def normale(theta,phi):
    # retourne le vecteur unitaire définit par (cos(theta)*sin(phi),sin(theta)*sin(phi),cos(phi))
    vN = np.array([cos(theta)*sin(phi),sin(theta)*sin(phi),cos(phi)])
    return vN.T

# Cellule 4 : Fonction tens_to_mat
def tens_to_mat(liste):
    if isinstance(liste,list):
        liste = np.array(liste)
    res = np.array([[liste[0],liste[3],liste[4]],
                    [liste[3],liste[1],liste[5]],
                    [liste[4],liste[5],liste[2]]])
    return res

# Cellule 5 : Fonction contTang
def contTang(tens,vN):
    # calcul le vecteur contrainte tangentiell sur une facette de normale vN
    M = tens_to_mat(tens)  # vecteur contrainte
    cont = M@vN  # contrainte normale
    cN = cont@vN
    contT = cont-cN*vN
    return contT

# Cellule 6 : Fonction hydro
def hydro(tens):
    # cette fonction doit retourner la pression hydrostatique associée à ce tenseur (c'est pour un instant du cycle !)
    p = (tens[0]+tens[1]+tens[2])/3
    return p

# Cellule 7 : Fonction genereTens
def genereTens(sigma1,omega,pasTemps,fin):
    tens = np.array([sigma1,0,0,0,0,0])
    for i in range(int(fin/pasTemps)):
        t = (i+1)*pasTemps
        ligne = np.array([sigma1*cos(omega*t),0,0,0,0,0])
        tens = np.vstack((tens, ligne))
    # omega est la pulsation, vous pouvez choisir 2*pi par exemple
    # sigma1 est fixe, par exemple 100 MPa
    # cette fonction doit générer une matrice de 6 colonnes, chaque ligne étant le tenseur à un instant du cycle, et de la forme [sigma1*cos(omega*t),0,0,0,0,0]
    return tens
def genereTensOrt(sigma1,omega,pasTemps,fin):
    tens = np.array([0,0,0,sigma1,0,0])
    for i in range(int(fin/pasTemps)):
        t = (i+1)*pasTemps
        ligne = np.array([0,0,0,sigma1*cos(omega*t),0,0])
        tens = np.vstack((tens, ligne))
    # omega est la pulsation, vous pouvez choisir 2*pi par exemple
    # sigma1 est fixe, par exemple 100 MPa
    # cette fonction doit générer une matrice de 6 colonnes, chaque ligne étant le tenseur à un instant du cycle, et de la forme [sigma1*cos(omega*t),0,0,0,0,0]
    return tens

# Cellule 8 : Test de genereTens
# genereTens(100,2*pi,0.01,1)

# Cellule 9 : Fonction amplitudeTangMax
def amplitudeTangMax(tens):
    # cette fonction doit retourner pour UN instant une liste de deux éléments : 
    # le premier élément est la valeur max_n (norme de contTang) et le deuxième les angles du plan associés
    # il faut balayer les facettes !
    maxi = 0
    theta = 0
    planMax = [0,0]
    phi = 0
    pasTheta = pi/180
    pasPhi = pi/180
    vect_norm = normale(theta,phi)
    
    for i in range(180+1):
        theta = i*pasTheta
        for j in range(180+1):
            phi = j*pasPhi
            # on construit le vecteur normal
            vect_norm = normale(theta,phi)
            # on calcule la contrainte tangentielle
            contT = contTang(tens,vect_norm)
            # on calcule sa norme
            norme = np.linalg.norm(contT)
            # si elle est plus grande que maxi, elle devient maxi
            if norme > maxi:
                maxi = norme
                planMax = [theta,phi]
            # on actualise planMax
    # on retourne [maxi,planMax]
    return [maxi,planMax]

# Cellule 10 : Fonction nuage
def nuage(sigma1,omega,pasTemps,fin):
    """
    Le but de la fonction est de tracer les contraintes tangentielles maximales en fonction de la 
    pression hydrostatique
    """
    points = np.array([0,0])
    tensTot = genereTens(sigma1,omega,pasTemps,fin)
    for t in range(int(fin/pasTemps)):
        tens = tensTot[t]
        cisMax,_ = amplitudeTangMax(tens)
        hydros = hydro(tens)
        ligne = np.array([hydros,cisMax])
        points = np.vstack((points, ligne))
    return points



def nuageOrt(sigma1,omega,pasTemps,fin):
    """
    Le but de la fonction est de tracer les contraintes tangentielles maximales en fonction de la 
    pression hydrostatique
    """
    points = np.array([0,0])
    tensTot = genereTensOrt(sigma1,omega,pasTemps,fin)
    for t in range(int(fin/pasTemps)):
        tens = tensTot[t]
        cisMax,_ = amplitudeTangMax(tens)
        hydros = hydro(tens)
        ligne = np.array([hydros,cisMax])
        points = np.vstack((points, ligne))
    return points

# Cellule 11 : Fonction traceNuage
def traceNuage(points):
    plt.scatter(points[:, 0], points[:, 1])
    plt.xlabel("pression hydrostatique")
    plt.ylabel("amplitude de cisaillement max")
    plt.title("Nuage de points")
    plt.show()

# Cellule 12 : Exécution et visualisation
# points = nuage(100,2*pi,0.01,1)
# traceNuage(points)