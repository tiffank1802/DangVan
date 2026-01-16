import numpy as np
import matplotlib.pyplot as plt
from math import *
import pytest

import old.deviatoire as dev
import old.versDV as dv

def test_hydro():
    # ce test vérifie que la fonction hydrostatique calcule correctement la pression hydrostatique
    tens=np.array([100,50,25,0,0,0])
    expected=(100+50+25)/3
    result=dv.hydro(tens)
    assert np.isclose(result,expected)

def test_mat_to_tens_dev():
    tens=dev.deviateur(np.array([100,50,0,0,0,0]))
    mat=dv.tens_to_mat(tens)
    tens_back=dv.mat_to_tens(mat)
    tens_back=np.delete(tens_back, 2)  # on enlève la troisième composante qui n'est pas dans le tenseur déviateur
    assert np.allclose(tens,tens_back)  # on enlève la troisième composante qui n'est pas dans le tenseur déviateur
def test_deviateur():
    # ce test vérifie que la fonction deviateur calcule correctement le tenseur déviateur
    tens=np.array([100,0,0,0,0,0])
    expected=dv.tens_to_mat(tens)-np.eye(3)*dv.hydro(tens)
    expected=dv.mat_to_tens(expected)
    expected=np.delete(expected, 2)  # on enlève la troisième composante qui n'est pas dans le tenseur déviateur
    result=dev.deviateur(tens)
    assert np.allclose(result,expected)
def test_deviateur_matrix():
    # ce test vérifie que la trace du tenseur déviateur est nulle
    tens=np.array([100,0,0,0,0,0])
    tens_dev=dev.deviateur(tens)
    mat=dv.tens_to_mat(tens_dev)
    diag_zero=np.sum(np.diag(mat))
    assert np.isclose(diag_zero,0)

def test_tens_to_mat_and_back():
    # ce test vérifie la conversion aller-retour entre tenseur et matrice
    tens=np.array([100,50,25,10,5,0])
    mat=dv.tens_to_mat(tens)
    tens_back=dv.mat_to_tens(mat)
    assert np.allclose(tens,tens_back)
# def test_tens_to_mat():
#     tens=np.array([100,50,25,10,5])
#     H=dv.hydro(tens)
#     expected=np.array([[H,tens]])

def test_CalculMatDev():
    # ce test vérifie que la fonction CalculMatDev calcule correctement la matrice des tenseurs déviateurs
    matTens=np.array([[100,0,0,0,0,0],[50,50,0,0,0,0]])
    expected=np.array([[2*100/3,-100/3,0,0,0],[50/3,50/3,0,0,0]])
    result=dev.CalculMatDev(matTens)
    assert np.allclose(result,expected)

def test_normeTresca():
    # ce test vérifie que la fonction normeTresca calcule correctement la norme de Tresca
    tens=np.array([100,0,0,0,0,0])
    expected=100
    result=dev.normeTresca(tens)
    assert np.isclose(result,expected)

def test_diametre_deux():
    # ce test vérifie que la fonction diametre calcule correctement le diamètre au sens de la norme de Tresca pour deux tenseurs
    matTens=np.array([[100,0,0,0,0,0],[50,0,0,0,0,0]])
    matDev=dev.CalculMatDev(matTens)
    expected=dev.normeTresca(matDev[0]-matDev[1]),matDev[0],matDev[1]
    result=dev.diametre(matTens)
    assert np.isclose(result[0],expected[0])
    assert np.allclose(result[1],expected[1])
    assert np.allclose(result[2],expected[2])
def test_diametre_multiple():
    # ce test vérifie que la fonction diametre calcule correctement le diamètre au sens de la norme de Tresca pour plusieurs tenseurs
    matTens=np.array([[100,0,0,0,0,0],[50,0,0,0,0,0],[0,0,0,0,0,0]])
    matDev=dev.CalculMatDev(matTens)
    expected=dev.normeTresca(matDev[0]-matDev[2]),matDev[0],matDev[2]
    result=dev.diametre(matTens)
    # assert np.isclose(result[0],expected[0])
    # assert np.allclose(result[1],expected[1])