"""Digitale Bildhauerei per Code: Formen als Distanzfelder (SDF), weich verschmolzen,
und ein Vernetzer (Surface Nets), der daraus dichte Meshes erzeugt.

  sdf.py     Grundformen (Kugel, Ellipsoid, Rundkegel, Loft, Box ...) und das Sculpt-Objekt,
             das Operationen (hinzufügen, abziehen, Kleidung, Verschiebung) nacheinander anwendet
  noise.py   Gradientenrauschen in numpy (für Falten, Risse, Löcher)
  mesher.py  dünn besetzte Auswertung in Blöcken + Surface Nets -> Blender-Mesh
"""
