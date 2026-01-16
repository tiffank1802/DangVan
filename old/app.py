import streamlit as st
import numpy as np
import matplotlib.pyplot as plt
import old.versDV as dv
import old.deviatoire as dev
from math import pi

st.title("Dang Van Fatigue Criterion Web App")

st.sidebar.header("Parameters")

sigma1 = st.sidebar.slider("Amplitude sigma1 (MPa)", 10, 200, 100)
omega = st.sidebar.slider("Frequency omega (rad/s)", 0.1, 10.0, 2*pi, step=0.1)
pasTemps = st.sidebar.slider("Time step pasTemps", 0.001, 0.1, 0.01, step=0.001)
fin = st.sidebar.slider("End time fin", 0.1, 2.0, 1.0, step=0.1)

if st.button("Compute and Plot"):
    with st.spinner("Computing..."):
        # Compute points for uniaxial
        points_uniaxial = dv.nuage(sigma1, omega, pasTemps, fin)
        # Compute points for torsion
        points_torsion = dv.nuageOrt(sigma1, omega, pasTemps, fin)

    fig, ax = plt.subplots()
    ax.scatter(points_uniaxial[:, 0], points_uniaxial[:, 1], label='Traction-Compression')
    ax.scatter(points_torsion[:, 0], points_torsion[:, 1], label='Torsion')
    ax.set_xlabel("Pression hydrostatique")
    ax.set_ylabel("Amplitude de cisaillement max")
    ax.set_title("Diagramme de Dang Van")
    ax.legend()
    ax.grid(True)
    st.pyplot(fig)

st.write("This app computes the Dang Van criterion for fatigue analysis under uniaxial and torsion loading.")