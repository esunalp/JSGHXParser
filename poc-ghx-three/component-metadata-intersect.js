export const INTERSECT_COMPONENTS = [
  {
    "category": "Intersect",
    "subcategory": "Shape",
    "name": "Split Brep Multiple",
    "nickname": "SplitMul",
    "guid": "03f22640-ff80-484e-bb53-a4025c5faa07",
    "description": "Split one brep with a bunch of others.",
    "inputs": [
      {
        "name": "Brep",
        "nickname": "B",
        "access": "item",
        "description": "Brep to split"
      },
      {
        "name": "Cutters",
        "nickname": "C",
        "access": "list",
        "description": "Cutting shapes"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "R",
        "access": "list",
        "description": "Brep fragments"
      }
    ]
  },
  {
    "category": "Intersect",
    "subcategory": "Physical",
    "name": "Curve | Self",
    "nickname": "CX",
    "guid": "0991ac99-6a0b-47a9-b07d-dd510ca57f0f",
    "description": "Solve all self intersection events for a curve.",
    "inputs": [
      {
        "name": "Curve",
        "nickname": "C",
        "access": "item",
        "description": "Curve for self-intersections"
      }
    ],
    "outputs": [
      {
        "name": "Points",
        "nickname": "P",
        "access": "list",
        "description": "Intersection events"
      },
      {
        "name": "Params",
        "nickname": "t",
        "access": "list",
        "description": "Parameters on curve"
      }
    ]
  },
  {
    "category": "Intersect",
    "subcategory": "Mathematical",
    "name": "Curve | Line",
    "nickname": "CLX",
    "guid": "0e3173b6-91c6-4845-a748-e45d4fdbc262",
    "description": "Solve intersection events for a curve and a line.",
    "inputs": [
      {
        "name": "Curve",
        "nickname": "C",
        "access": "item",
        "description": "Curve to intersect"
      },
      {
        "name": "Line",
        "nickname": "L",
        "access": "item",
        "description": "Line to intersect with"
      }
    ],
    "outputs": [
      {
        "name": "Points",
        "nickname": "P",
        "access": "list",
        "description": "Intersection events"
      },
      {
        "name": "Params",
        "nickname": "t",
        "access": "list",
        "description": "Parameters on curve"
      },
      {
        "name": "Count",
        "nickname": "N",
        "access": "item",
        "description": "Number of intersection events"
      }
    ]
  },
  {
    "category": "Intersect",
    "subcategory": "Shape",
    "name": "Region Slits",
    "nickname": "RSlits",
    "guid": "0feeeaca-8f1f-4d7c-a24a-8e7dd68604a2",
    "description": "Add slits to a collection of intersecting planar regions",
    "inputs": [
      {
        "name": "Regions",
        "nickname": "R",
        "access": "list",
        "description": "Planar regions to intersect"
      },
      {
        "name": "Width",
        "nickname": "W",
        "access": "item",
        "description": "Width of slits"
      },
      {
        "name": "Gap",
        "nickname": "G",
        "access": "item",
        "description": "Additional gap size at slit meeting points"
      }
    ],
    "outputs": [
      {
        "name": "Regions",
        "nickname": "R",
        "access": "tree",
        "description": "Regions with slits"
      },
      {
        "name": "Topology",
        "nickname": "T",
        "access": "tree",
        "description": "Slit topology"
      }
    ]
  },
  {
    "category": "Intersect",
    "subcategory": "Shape",
    "name": "Solid Union",
    "nickname": "SUnion",
    "guid": "10434a15-da85-4281-bb64-a2b3a995b9c6",
    "description": "Perform a solid union on a set of Breps.",
    "inputs": [
      {
        "name": "Breps",
        "nickname": "B",
        "access": "list",
        "description": "Breps to union"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "R",
        "access": "list",
        "description": "Union result"
      }
    ]
  },
  {
    "category": "Intersect",
    "subcategory": "Shape",
    "name": "Region Union",
    "nickname": "RUnion",
    "guid": "1222394f-0d33-4f31-9101-7281bde89fe5",
    "description": "Union of a set of planar closed curves (regions)",
    "inputs": [
      {
        "name": "Curves",
        "nickname": "C",
        "access": "list",
        "description": "Curves for boolean union operation"
      },
      {
        "name": "Plane",
        "nickname": "P",
        "access": "item",
        "description": "Optional plane for boolean solution"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "R",
        "access": "list",
        "description": "Result outlines of boolean union"
      }
    ]
  },
  {
    "category": "Intersect",
    "subcategory": "Physical",
    "name": "Mesh | Curve",
    "nickname": "MCX",
    "guid": "19632848-4b95-4e5e-9e86-b79b47987a46",
    "description": "Mesh Curve intersection",
    "inputs": [
      {
        "name": "Mesh",
        "nickname": "M",
        "access": "item",
        "description": "Mesh to intersect"
      },
      {
        "name": "Curve",
        "nickname": "C",
        "access": "item",
        "description": "Curve to intersect with"
      }
    ],
    "outputs": [
      {
        "name": "Points",
        "nickname": "X",
        "access": "list",
        "description": "Intersection points"
      },
      {
        "name": "Faces",
        "nickname": "F",
        "access": "list",
        "description": "Intersection face index for each point"
      }
    ]
  },
  {
    "category": "Intersect",
    "subcategory": "Physical",
    "name": "Brep | Curve",
    "nickname": "BCX",
    "guid": "20ef81e8-df15-4a0c-acf1-993a7607cafb",
    "description": "Solve intersection events for a Brep and a curve.",
    "inputs": [
      {
        "name": "Brep",
        "nickname": "B",
        "access": "item",
        "description": "Base Brep"
      },
      {
        "name": "Curve",
        "nickname": "C",
        "access": "item",
        "description": "Intersection curve"
      }
    ],
    "outputs": [
      {
        "name": "Curves",
        "nickname": "C",
        "access": "list",
        "description": "Intersection overlap curves"
      },
      {
        "name": "Points",
        "nickname": "P",
        "access": "list",
        "description": "Intersection points"
      }
    ]
  },
  {
    "category": "Intersect",
    "subcategory": "Physical",
    "name": "Collision Many|Many",
    "nickname": "ColMM",
    "guid": "2168853c-acd8-4a63-9c9b-ecde9e239eae",
    "description": "Test for many|many collision between objects",
    "inputs": [
      {
        "name": "Colliders",
        "nickname": "C",
        "access": "list",
        "description": "Objects for collision"
      }
    ],
    "outputs": [
      {
        "name": "Collision",
        "nickname": "C",
        "access": "list",
        "description": "True if object at this index collides with any of the other objects"
      },
      {
        "name": "Indices",
        "nickname": "I",
        "access": "list",
        "description": "Index of object in set which collided with the object at this index"
      }
    ]
  },
  {
    "category": "Intersect",
    "subcategory": "Physical",
    "name": "Mesh | Mesh",
    "nickname": "MMX",
    "guid": "21b6a605-9568-4bf8-acc1-631565d609d7",
    "description": "Mesh Mesh intersection",
    "inputs": [
      {
        "name": "Mesh A",
        "nickname": "A",
        "access": "item",
        "description": "First mesh"
      },
      {
        "name": "Mesh B",
        "nickname": "B",
        "access": "item",
        "description": "Second mesh"
      }
    ],
    "outputs": [
      {
        "name": "Intersections",
        "nickname": "X",
        "access": "list",
        "description": "Intersection polylines"
      }
    ]
  },
  {
    "category": "Intersect",
    "subcategory": "Mathematical",
    "name": "Contour (ex)",
    "nickname": "Contour",
    "guid": "246cda78-5e88-4087-ba09-ae082bbc4af8",
    "description": "Create a set of Brep or Mesh contours",
    "inputs": [
      {
        "name": "Shape",
        "nickname": "S",
        "access": "item",
        "description": "Brep or Mesh to contour"
      },
      {
        "name": "Plane",
        "nickname": "P",
        "access": "item",
        "description": "Base plane for contours"
      },
      {
        "name": "Offsets",
        "nickname": "O",
        "access": "list",
        "description": "Contour offsets from base plane (if omitted, you must specify distances instead)"
      },
      {
        "name": "Distances",
        "nickname": "D",
        "access": "list",
        "description": "Distances between contours (if omitted, you must specify offset instead)"
      }
    ],
    "outputs": [
      {
        "name": "Contours",
        "nickname": "C",
        "access": "tree",
        "description": "Resulting contours (grouped by section)"
      }
    ]
  },
  {
    "category": "Intersect",
    "subcategory": "Region",
    "name": "Trim with Regions",
    "nickname": "Trim",
    "guid": "26949c81-9b50-43b7-ac49-3203deb6eec7",
    "description": "Trim a curve with multiple regions.",
    "inputs": [
      {
        "name": "Curve",
        "nickname": "C",
        "access": "item",
        "description": "Curve to trim"
      },
      {
        "name": "Regions",
        "nickname": "R",
        "access": "list",
        "description": "Regions to trim against"
      },
      {
        "name": "Plane",
        "nickname": "P",
        "access": "item",
        "description": "Optional solution plane. If omitted the curve best-fit plane is used."
      }
    ],
    "outputs": [
      {
        "name": "Inside",
        "nickname": "Ci",
        "access": "list",
        "description": "Split curves inside the regions"
      },
      {
        "name": "Outside",
        "nickname": "Co",
        "access": "list",
        "description": "Split curves outside the regions"
      }
    ]
  },
  {
    "category": "Intersect",
    "subcategory": "Mathematical",
    "name": "Plane | Plane",
    "nickname": "PPX",
    "guid": "290cf9c4-0711-4704-851e-4c99e3343ac5",
    "description": "Solve the intersection event of two planes.",
    "inputs": [
      {
        "name": "Plane A",
        "nickname": "A",
        "access": "item",
        "description": "First plane"
      },
      {
        "name": "Plane B",
        "nickname": "B",
        "access": "item",
        "description": "Second plane"
      }
    ],
    "outputs": [
      {
        "name": "Line",
        "nickname": "L",
        "access": "item",
        "description": "Intersection line"
      }
    ]
  },
  {
    "category": "Intersect",
    "subcategory": "Shape",
    "name": "Box Slits",
    "nickname": "Slits",
    "guid": "2d3b6ef3-5c26-4e2f-bcb3-8ffb9fb0f7c3",
    "description": "Add slits to a collection of intersecting boxes",
    "inputs": [
      {
        "name": "Boxes",
        "nickname": "B",
        "access": "list",
        "description": "Boxes to intersect"
      },
      {
        "name": "Gap",
        "nickname": "G",
        "access": "item",
        "description": "Additional gap width"
      }
    ],
    "outputs": [
      {
        "name": "Breps",
        "nickname": "B",
        "access": "tree",
        "description": "Boxes with slits"
      },
      {
        "name": "Topology",
        "nickname": "T",
        "access": "tree",
        "description": "Slit topology"
      }
    ]
  },
  {
    "category": "Intersect",
    "subcategory": "Region",
    "name": "Trim with Region",
    "nickname": "Trim",
    "guid": "3092caf0-7cf9-4885-bcc0-e635d878832a",
    "description": "Trim a curve with a region.",
    "inputs": [
      {
        "name": "Curve",
        "nickname": "C",
        "access": "item",
        "description": "Curve to trim"
      },
      {
        "name": "Region",
        "nickname": "R",
        "access": "item",
        "description": "Region to trim against"
      },
      {
        "name": "Plane",
        "nickname": "P",
        "access": "item",
        "description": "Optional solution plane. If omitted the curve best-fit plane is used."
      }
    ],
    "outputs": [
      {
        "name": "Inside",
        "nickname": "Ci",
        "access": "list",
        "description": "Split curves inside the region"
      },
      {
        "name": "Outside",
        "nickname": "Co",
        "access": "list",
        "description": "Split curves outside the region"
      }
    ]
  },
  {
    "category": "Intersect",
    "subcategory": "Mathematical",
    "name": "Contour",
    "nickname": "Contour",
    "guid": "3b112fb6-3eba-42d2-ba75-0f903c18faab",
    "description": "Create a set of Brep or Mesh contours",
    "inputs": [
      {
        "name": "Shape",
        "nickname": "S",
        "access": "item",
        "description": "Brep or Mesh to contour"
      },
      {
        "name": "Point",
        "nickname": "P",
        "access": "item",
        "description": "Contour start point"
      },
      {
        "name": "Direction",
        "nickname": "N",
        "access": "item",
        "description": "Contour normal direction"
      },
      {
        "name": "Distance",
        "nickname": "D",
        "access": "item",
        "description": "Distance between contours"
      }
    ],
    "outputs": [
      {
        "name": "Contours",
        "nickname": "C",
        "access": "tree",
        "description": "Resulting contours (grouped by section)"
      }
    ]
  },
  {
    "category": "Intersect",
    "subcategory": "Mathematical",
    "name": "Mesh | Plane",
    "nickname": "Sec",
    "guid": "3b1ae469-0e9b-461d-8c30-fa5a7de8b7a9",
    "description": "Solve intersection events for a Mesh and a Plane (otherwise known as section).",
    "inputs": [
      {
        "name": "Mesh",
        "nickname": "M",
        "access": "item",
        "description": "Base Mesh"
      },
      {
        "name": "Plane",
        "nickname": "P",
        "access": "item",
        "description": "Section plane"
      }
    ],
    "outputs": [
      {
        "name": "Curves",
        "nickname": "C",
        "access": "list",
        "description": "Section polylines"
      }
    ]
  },
  {
    "category": "Intersect",
    "subcategory": "Region",
    "name": "Trim with Brep",
    "nickname": "Trim",
    "guid": "3eba04bc-00e8-416d-b58f-a3dc8b3e22e2",
    "description": "Trim a curve with a Brep.",
    "inputs": [
      {
        "name": "Curve",
        "nickname": "C",
        "access": "item",
        "description": "Curve to trim"
      },
      {
        "name": "Brep",
        "nickname": "B",
        "access": "item",
        "description": "Brep to trim against"
      }
    ],
    "outputs": [
      {
        "name": "Inside",
        "nickname": "Ci",
        "access": "list",
        "description": "Split curves inside the Brep"
      },
      {
        "name": "Outside",
        "nickname": "Co",
        "access": "list",
        "description": "Split curves outside the Brep"
      }
    ]
  },
  {
    "category": "Intersect",
    "subcategory": "Physical",
    "name": "Clash",
    "nickname": "Clash",
    "guid": "4439a51b-8d24-4924-b8e2-f77e7f8f5bec",
    "description": "Perform clash analysis on a set of shapes.",
    "inputs": [
      {
        "name": "First Set",
        "nickname": "A",
        "access": "list",
        "description": "First set of shapes"
      },
      {
        "name": "Second Set",
        "nickname": "B",
        "access": "list",
        "description": "Second set of shapes"
      },
      {
        "name": "Distance",
        "nickname": "D",
        "access": "item",
        "description": "Distance tolerance for clash detection"
      },
      {
        "name": "Result Limit",
        "nickname": "L",
        "access": "item",
        "description": "Maximum number of results to search for."
      }
    ],
    "outputs": [
      {
        "name": "Clash Count",
        "nickname": "N",
        "access": "item",
        "description": "Number of clashes found"
      },
      {
        "name": "Clash Points",
        "nickname": "P",
        "access": "list",
        "description": "Collection of clashing points."
      },
      {
        "name": "Clash Radii",
        "nickname": "R",
        "access": "list",
        "description": "Collection of clashing radii (one for each point)."
      },
      {
        "name": "First Index",
        "nickname": "i",
        "access": "list",
        "description": "Index of clashing mesh in first set."
      },
      {
        "name": "Second index",
        "nickname": "j",
        "access": "list",
        "description": "Index of clashing mesh in second set."
      }
    ]
  },
  {
    "category": "Intersect",
    "subcategory": "Shape",
    "name": "Region Intersection",
    "nickname": "RInt",
    "guid": "477c2e7b-c5e5-421e-b8b2-ba60cdf5398b",
    "description": "Intersection between two sets of planar closed curves (regions)",
    "inputs": [
      {
        "name": "Curves A",
        "nickname": "A",
        "access": "list",
        "description": "First set of regions."
      },
      {
        "name": "Curves B",
        "nickname": "B",
        "access": "list",
        "description": "Second set of regions."
      },
      {
        "name": "Plane",
        "nickname": "P",
        "access": "item",
        "description": "Optional plane for boolean solution"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "R",
        "access": "list",
        "description": "Result outlines of boolean intersection (A and B)"
      }
    ]
  },
  {
    "category": "Intersect",
    "subcategory": "Region",
    "name": "Split with Brep",
    "nickname": "Split",
    "guid": "4bdc2eb0-24ed-4c90-a27b-a32db069eaef",
    "description": "Split a curve with a Brep.",
    "inputs": [
      {
        "name": "Curve",
        "nickname": "C",
        "access": "item",
        "description": "Curve to split"
      },
      {
        "name": "Brep",
        "nickname": "B",
        "access": "item",
        "description": "Brep to split with"
      }
    ],
    "outputs": [
      {
        "name": "Curve",
        "nickname": "C",
        "access": "list",
        "description": "Split curves"
      },
      {
        "name": "Points",
        "nickname": "P",
        "access": "list",
        "description": "Split points"
      }
    ]
  },
  {
    "category": "Intersect",
    "subcategory": "Mathematical",
    "name": "Mesh | Ray",
    "nickname": "MeshRay",
    "guid": "4c02a168-9aba-4f42-8951-2719f24d391f",
    "description": "Intersect a mesh with a semi-infinite ray",
    "inputs": [
      {
        "name": "Mesh",
        "nickname": "M",
        "access": "item",
        "description": "Mesh to intersect"
      },
      {
        "name": "Point",
        "nickname": "P",
        "access": "item",
        "description": "Ray start point"
      },
      {
        "name": "Direction",
        "nickname": "D",
        "access": "item",
        "description": "Ray direction"
      }
    ],
    "outputs": [
      {
        "name": "Point",
        "nickname": "X",
        "access": "item",
        "description": "First intersection point"
      },
      {
        "name": "Hit",
        "nickname": "H",
        "access": "item",
        "description": "Boolean indicating hit or miss"
      }
    ]
  },
  {
    "category": "Intersect",
    "subcategory": "Shape",
    "name": "Mesh Difference",
    "nickname": "MDif",
    "guid": "4f3147f4-9fcd-4a7e-be0e-b1841caa5f97",
    "description": "Perform a solid difference on two sets of meshes",
    "inputs": [
      {
        "name": "Meshes A",
        "nickname": "A",
        "access": "list",
        "description": "First mesh set"
      },
      {
        "name": "Meshes B",
        "nickname": "B",
        "access": "list",
        "description": "Second mesh set"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "R",
        "access": "list",
        "description": "Difference result of A-B"
      }
    ]
  },
  {
    "category": "Intersect",
    "subcategory": "Mathematical",
    "name": "Brep | Plane",
    "nickname": "Sec",
    "guid": "4fe828e8-fa95-4cc5-9a8c-c33856ecc783",
    "description": "Solve intersection events for a Brep and a plane (otherwise known as section).",
    "inputs": [
      {
        "name": "Brep",
        "nickname": "B",
        "access": "item",
        "description": "Base Brep"
      },
      {
        "name": "Plane",
        "nickname": "P",
        "access": "item",
        "description": "Section plane"
      }
    ],
    "outputs": [
      {
        "name": "Curves",
        "nickname": "C",
        "access": "list",
        "description": "Section curves"
      },
      {
        "name": "Points",
        "nickname": "P",
        "access": "list",
        "description": "Section points"
      }
    ]
  },
  {
    "category": "Intersect",
    "subcategory": "Shape",
    "name": "Solid Intersection",
    "nickname": "SInt",
    "guid": "5723c845-cafc-442d-a667-8c76532845e6",
    "description": "Perform a solid intersection on two Brep sets.",
    "inputs": [
      {
        "name": "Breps A",
        "nickname": "A",
        "access": "list",
        "description": "First Brep set"
      },
      {
        "name": "Breps B",
        "nickname": "B",
        "access": "list",
        "description": "Second Brep set"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "R",
        "access": "list",
        "description": "Intersection result"
      }
    ]
  },
  {
    "category": "Intersect",
    "subcategory": "Region",
    "name": "Split with Breps",
    "nickname": "Split",
    "guid": "5b742537-9bcb-4f06-9613-866da5bf845e",
    "description": "Split a curve with multiple Breps.",
    "inputs": [
      {
        "name": "Curve",
        "nickname": "C",
        "access": "item",
        "description": "Curve to trim"
      },
      {
        "name": "Brep",
        "nickname": "B",
        "access": "list",
        "description": "Brep to trim against"
      }
    ],
    "outputs": [
      {
        "name": "Curve",
        "nickname": "C",
        "access": "list",
        "description": "Split curves"
      },
      {
        "name": "Points",
        "nickname": "P",
        "access": "list",
        "description": "Split points"
      }
    ]
  },
  {
    "category": "Intersect",
    "subcategory": "Physical",
    "name": "Surface | Curve",
    "nickname": "SCX",
    "guid": "68546dd0-aa82-471c-87e9-81cb16ac50ed",
    "description": "Solve intersection events for a surface and a curve.",
    "inputs": [
      {
        "name": "Surface",
        "nickname": "S",
        "access": "item",
        "description": "Base surface"
      },
      {
        "name": "Curve",
        "nickname": "C",
        "access": "item",
        "description": "Intersection curve"
      }
    ],
    "outputs": [
      {
        "name": "Curves",
        "nickname": "C",
        "access": "list",
        "description": "Intersection overlap curves"
      },
      {
        "name": "Points",
        "nickname": "P",
        "access": "list",
        "description": "Intersection points"
      },
      {
        "name": "UV Points",
        "nickname": "uv",
        "access": "list",
        "description": "Surface {uv} coordinates at intersection events"
      },
      {
        "name": "Normals",
        "nickname": "N",
        "access": "list",
        "description": "Surface normal vector at intersection events"
      },
      {
        "name": "Parameters",
        "nickname": "t",
        "access": "list",
        "description": "Curve parameter at intersection events"
      },
      {
        "name": "Tangents",
        "nickname": "T",
        "access": "list",
        "description": "Curve tangent vector at intersection events"
      }
    ]
  },
  {
    "category": "Intersect",
    "subcategory": "Mathematical",
    "name": "Line | Line",
    "nickname": "LLX",
    "guid": "6d4b82a7-8c1d-4bec-af7b-ca321ba4beb1",
    "description": "Solve intersection events for two lines.",
    "inputs": [
      {
        "name": "Line 1",
        "nickname": "A",
        "access": "item",
        "description": "First line for intersection"
      },
      {
        "name": "Line 2",
        "nickname": "B",
        "access": "item",
        "description": "Second line for intersection"
      }
    ],
    "outputs": [
      {
        "name": "Param A",
        "nickname": "tA",
        "access": "item",
        "description": "Parameter on line A"
      },
      {
        "name": "Param B",
        "nickname": "tB",
        "access": "item",
        "description": "Parameter on line B"
      },
      {
        "name": "Point A",
        "nickname": "pA",
        "access": "item",
        "description": "Point on line A"
      },
      {
        "name": "Point B",
        "nickname": "pB",
        "access": "item",
        "description": "Point on line B"
      }
    ]
  },
  {
    "category": "Intersect",
    "subcategory": "Mathematical",
    "name": "Line | Plane",
    "nickname": "PLX",
    "guid": "75d0442c-1aa3-47cf-bd94-457b42c16e9f",
    "description": "Solve intersection event for a line and a plane.",
    "inputs": [
      {
        "name": "Line",
        "nickname": "L",
        "access": "item",
        "description": "Base line"
      },
      {
        "name": "Plane",
        "nickname": "P",
        "access": "item",
        "description": "Intersection plane"
      }
    ],
    "outputs": [
      {
        "name": "Point",
        "nickname": "P",
        "access": "item",
        "description": "Intersection event"
      },
      {
        "name": "Param L",
        "nickname": "t",
        "access": "item",
        "description": "Parameter {t} on infinite line"
      },
      {
        "name": "Param P",
        "nickname": "uv",
        "access": "item",
        "description": "Parameter {uv} on plane"
      }
    ]
  },
  {
    "category": "Intersect",
    "subcategory": "Mathematical",
    "name": "IsoVist Ray",
    "nickname": "IVRay",
    "guid": "769f5b35-1780-4823-b593-118ecc3560e0",
    "description": "Compute a single isovist sample at a location",
    "inputs": [
      {
        "name": "Sample",
        "nickname": "S",
        "access": "item",
        "description": "Sampling ray"
      },
      {
        "name": "Radius",
        "nickname": "R",
        "access": "item",
        "description": "Sample radius"
      },
      {
        "name": "Obstacles",
        "nickname": "O",
        "access": "list",
        "description": "Obstacle outlines"
      }
    ],
    "outputs": [
      {
        "name": "Point",
        "nickname": "P",
        "access": "item",
        "description": "Intersection point of the sample ray with the obstacles"
      },
      {
        "name": "Distance",
        "nickname": "D",
        "access": "item",
        "description": "Distance from ray start to intersection point"
      },
      {
        "name": "Hit",
        "nickname": "H",
        "access": "item",
        "description": "Hit flag indicating whether the ray hit any of the obstacles"
      }
    ]
  },
  {
    "category": "Intersect",
    "subcategory": "Physical",
    "name": "Surface Split",
    "nickname": "SrfSplit",
    "guid": "7db14002-c09c-4d7b-9f80-e4e2b00dfa1d",
    "description": "Split a surface with a bunch of curves.",
    "inputs": [
      {
        "name": "Surface",
        "nickname": "S",
        "access": "item",
        "description": "Base surface"
      },
      {
        "name": "Curves",
        "nickname": "C",
        "access": "list",
        "description": "Splitting curves"
      }
    ],
    "outputs": [
      {
        "name": "Fragments",
        "nickname": "F",
        "access": "list",
        "description": "Splitting fragments"
      }
    ]
  },
  {
    "category": "Intersect",
    "subcategory": "Mathematical",
    "name": "Plane Region",
    "nickname": "PlReg",
    "guid": "80e3614a-25ae-43e7-bb0a-760e68ade864",
    "description": "Create a bounded region from intersecting planes.",
    "inputs": [
      {
        "name": "Plane",
        "nickname": "P",
        "access": "item",
        "description": "Region plane and origin"
      },
      {
        "name": "Bounds",
        "nickname": "B",
        "access": "list",
        "description": "Region bounding planes"
      }
    ],
    "outputs": [
      {
        "name": "Region",
        "nickname": "R",
        "access": "item",
        "description": "Bounded region"
      }
    ]
  },
  {
    "category": "Intersect",
    "subcategory": "Physical",
    "name": "Curve | Curve",
    "nickname": "CCX",
    "guid": "84627490-0fb2-4498-8138-ad134ee4cb36",
    "description": "Solve intersection events for two curves.",
    "inputs": [
      {
        "name": "Curve A",
        "nickname": "A",
        "access": "item",
        "description": "First curve"
      },
      {
        "name": "Curve B",
        "nickname": "B",
        "access": "item",
        "description": "Second curve"
      }
    ],
    "outputs": [
      {
        "name": "Points",
        "nickname": "P",
        "access": "list",
        "description": "Intersection events"
      },
      {
        "name": "Params A",
        "nickname": "tA",
        "access": "list",
        "description": "Parameters on first curve"
      },
      {
        "name": "Params B",
        "nickname": "tB",
        "access": "list",
        "description": "Parameters on second curve"
      }
    ]
  },
  {
    "category": "Intersect",
    "subcategory": "Shape",
    "name": "Mesh Union",
    "nickname": "MUnion",
    "guid": "88060a82-0bf7-46bb-9af8-bdc860cf7e1d",
    "description": "Perform a solid union on a set of meshes",
    "inputs": [
      {
        "name": "Meshes",
        "nickname": "M",
        "access": "list",
        "description": "Meshes to union"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "R",
        "access": "list",
        "description": "Mesh solid union result"
      }
    ]
  },
  {
    "category": "Intersect",
    "subcategory": "Physical",
    "name": "Brep | Brep",
    "nickname": "BBX",
    "guid": "904e4b56-484a-4814-b35f-aa4baf362117",
    "description": "Solve intersection events for two Breps.",
    "inputs": [
      {
        "name": "Brep A",
        "nickname": "A",
        "access": "item",
        "description": "First Brep"
      },
      {
        "name": "Brep B",
        "nickname": "B",
        "access": "item",
        "description": "Second Brep"
      }
    ],
    "outputs": [
      {
        "name": "Curves",
        "nickname": "C",
        "access": "list",
        "description": "Intersection curves"
      },
      {
        "name": "Points",
        "nickname": "P",
        "access": "list",
        "description": "Intersection points"
      }
    ]
  },
  {
    "category": "Intersect",
    "subcategory": "Region",
    "name": "Trim with Breps",
    "nickname": "Trim",
    "guid": "916e7ebc-524c-47ce-8936-e50a09a7b43c",
    "description": "Trim a curve with multiple Breps.",
    "inputs": [
      {
        "name": "Curve",
        "nickname": "C",
        "access": "item",
        "description": "Curve to trim"
      },
      {
        "name": "Brep",
        "nickname": "B",
        "access": "list",
        "description": "Breps to trim against"
      }
    ],
    "outputs": [
      {
        "name": "Inside",
        "nickname": "Ci",
        "access": "list",
        "description": "Split curves on the inside of the trimming Breps"
      },
      {
        "name": "Outside",
        "nickname": "Co",
        "access": "list",
        "description": "Split curves on the outside of the trimming Breps"
      }
    ]
  },
  {
    "category": "Intersect",
    "subcategory": "Physical",
    "name": "Multiple Curves",
    "nickname": "MCX",
    "guid": "931e6030-ccb3-4a7b-a89a-99dcce8770cd",
    "description": "Solve intersection events for multiple curves.",
    "inputs": [
      {
        "name": "Curves",
        "nickname": "C",
        "access": "list",
        "description": "Curves to intersect"
      }
    ],
    "outputs": [
      {
        "name": "Points",
        "nickname": "P",
        "access": "list",
        "description": "Intersection events"
      },
      {
        "name": "Index A",
        "nickname": "iA",
        "access": "list",
        "description": "Index of first intersection curve"
      },
      {
        "name": "Index B",
        "nickname": "iB",
        "access": "list",
        "description": "Index of second intersection curve"
      },
      {
        "name": "Param A",
        "nickname": "tA",
        "access": "list",
        "description": "Parameter on first curve"
      },
      {
        "name": "Param B",
        "nickname": "tB",
        "access": "list",
        "description": "Parameter on second curve"
      }
    ]
  },
  {
    "category": "Intersect",
    "subcategory": "Mathematical",
    "name": "Curve | Line",
    "nickname": "CLX",
    "guid": "9396be03-8159-43bf-b3e7-2c86c8d04fc0",
    "description": "Solve intersection events for a curve and a line.",
    "inputs": [
      {
        "name": "Curve",
        "nickname": "C",
        "access": "item",
        "description": "Curve to intersect"
      },
      {
        "name": "Line",
        "nickname": "L",
        "access": "item",
        "description": "Line to intersect with"
      },
      {
        "name": "First",
        "nickname": "F",
        "access": "item",
        "description": "Limit to first intersection only"
      }
    ],
    "outputs": [
      {
        "name": "Points",
        "nickname": "P",
        "access": "list",
        "description": "Intersection events"
      },
      {
        "name": "Params",
        "nickname": "t",
        "access": "list",
        "description": "Parameters on curve"
      },
      {
        "name": "Count",
        "nickname": "N",
        "access": "item",
        "description": "Number of intersection events"
      }
    ]
  },
  {
    "category": "Intersect",
    "subcategory": "Mathematical",
    "name": "IsoVist Ray",
    "nickname": "IVRay",
    "guid": "93d0dcbc-6207-4745-aaf7-fe57a880f959",
    "description": "Compute a single isovist sample at a location",
    "inputs": [
      {
        "name": "Sample",
        "nickname": "S",
        "access": "item",
        "description": "Sampling ray"
      },
      {
        "name": "Radius",
        "nickname": "R",
        "access": "item",
        "description": "Sample radius"
      },
      {
        "name": "Obstacles",
        "nickname": "O",
        "access": "list",
        "description": "Obstacle outlines (curves, planes, meshes and breps are allowed)"
      }
    ],
    "outputs": [
      {
        "name": "Point",
        "nickname": "P",
        "access": "item",
        "description": "Intersection point of the sample ray with the obstacles"
      },
      {
        "name": "Distance",
        "nickname": "D",
        "access": "item",
        "description": "Distance from ray start to intersection point"
      },
      {
        "name": "Index",
        "nickname": "I",
        "access": "item",
        "description": "Obstacle index for hit, or -1 if no obstacle was hit"
      }
    ]
  },
  {
    "category": "Intersect",
    "subcategory": "Shape",
    "name": "Mesh Intersection",
    "nickname": "MInt",
    "guid": "95aef4f6-66fc-477e-b8f8-32395a837831",
    "description": "Perform a solid intersection on a set of meshes",
    "inputs": [
      {
        "name": "Meshes A",
        "nickname": "A",
        "access": "list",
        "description": "First mesh set"
      },
      {
        "name": "Meshes B",
        "nickname": "B",
        "access": "list",
        "description": "Second mesh set"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "R",
        "access": "list",
        "description": "Intersection result of A&B"
      }
    ]
  },
  {
    "category": "Intersect",
    "subcategory": "Mathematical",
    "name": "Surface | Line",
    "nickname": "SLX",
    "guid": "a834e823-ae01-44d8-9066-c138eeb6f391",
    "description": "Solve intersection events for a surface and a line.",
    "inputs": [
      {
        "name": "Surface",
        "nickname": "S",
        "access": "item",
        "description": "Base surface"
      },
      {
        "name": "Line",
        "nickname": "L",
        "access": "item",
        "description": "Intersection line"
      }
    ],
    "outputs": [
      {
        "name": "Curves",
        "nickname": "C",
        "access": "list",
        "description": "Intersection overlap curves"
      },
      {
        "name": "Points",
        "nickname": "P",
        "access": "list",
        "description": "Intersection points"
      },
      {
        "name": "UV Points",
        "nickname": "uv",
        "access": "list",
        "description": "Surface {uv} coordinates at intersection events"
      },
      {
        "name": "Normal",
        "nickname": "N",
        "access": "list",
        "description": "Surface normal vector at intersection events"
      }
    ]
  },
  {
    "category": "Intersect",
    "subcategory": "Shape",
    "name": "Mesh Split",
    "nickname": "MSplit",
    "guid": "afbf2fe0-4965-48d2-8470-9e991540093b",
    "description": "Mesh Mesh split",
    "inputs": [
      {
        "name": "Mesh",
        "nickname": "M",
        "access": "item",
        "description": "Mesh to split"
      },
      {
        "name": "Splitters",
        "nickname": "S",
        "access": "list",
        "description": "Meshes to split with"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "R",
        "access": "list",
        "description": "Result of mesh split"
      }
    ]
  },
  {
    "category": "Intersect",
    "subcategory": "Shape",
    "name": "Boundary Volume",
    "nickname": "BVol",
    "guid": "b57bf805-046a-4360-ad76-51aeddfe9720",
    "description": "Create a closed polysurface from boundary surfaces",
    "inputs": [
      {
        "name": "Boundaries",
        "nickname": "B",
        "access": "list",
        "description": "Boundary surfaces"
      }
    ],
    "outputs": [
      {
        "name": "Solid",
        "nickname": "S",
        "access": "item",
        "description": "Solid volume"
      }
    ]
  },
  {
    "category": "Intersect",
    "subcategory": "Mathematical",
    "name": "Curve | Plane",
    "nickname": "PCX",
    "guid": "b7c12ed1-b09a-4e15-996f-3fa9f3f16b1c",
    "description": "Solve intersection events for a curve and a plane.",
    "inputs": [
      {
        "name": "Curve",
        "nickname": "C",
        "access": "item",
        "description": "Base curve"
      },
      {
        "name": "Plane",
        "nickname": "P",
        "access": "item",
        "description": "Intersection plane"
      }
    ],
    "outputs": [
      {
        "name": "Points",
        "nickname": "P",
        "access": "list",
        "description": "Intersection events"
      },
      {
        "name": "Params C",
        "nickname": "t",
        "access": "list",
        "description": "Parameters {t} on curve"
      },
      {
        "name": "Params P",
        "nickname": "uv",
        "access": "list",
        "description": "Parameters {uv} on plane"
      }
    ]
  },
  {
    "category": "Intersect",
    "subcategory": "Physical",
    "name": "Collision One|Many",
    "nickname": "ColOM",
    "guid": "bb6c6501-0500-4678-859b-b838348981d1",
    "description": "Test for one|many collision between objects",
    "inputs": [
      {
        "name": "Collider",
        "nickname": "C",
        "access": "item",
        "description": "Object for collision"
      },
      {
        "name": "Obstacles",
        "nickname": "O",
        "access": "list",
        "description": "Obstacles for collision"
      }
    ],
    "outputs": [
      {
        "name": "Collision",
        "nickname": "C",
        "access": "item",
        "description": "True if objects collides with any of the obstacles"
      },
      {
        "name": "Index",
        "nickname": "I",
        "access": "item",
        "description": "Index of first obstacle that collides with the object"
      }
    ]
  },
  {
    "category": "Intersect",
    "subcategory": "Mathematical",
    "name": "IsoVist",
    "nickname": "IVist",
    "guid": "c08ac8f7-cf90-4cdb-9862-2ba66b8408ef",
    "description": "Compute an isovist sampling at a location",
    "inputs": [
      {
        "name": "Plane",
        "nickname": "P",
        "access": "item",
        "description": "Sampling plane and origin"
      },
      {
        "name": "Count",
        "nickname": "N",
        "access": "item",
        "description": "Sample count"
      },
      {
        "name": "Radius",
        "nickname": "R",
        "access": "item",
        "description": "Sample radius"
      },
      {
        "name": "Obstacles",
        "nickname": "O",
        "access": "list",
        "description": "Obstacle outlines"
      }
    ],
    "outputs": [
      {
        "name": "Points",
        "nickname": "P",
        "access": "list",
        "description": "Intersection points of the sample rays with the obstacles"
      },
      {
        "name": "Distance",
        "nickname": "D",
        "access": "list",
        "description": "List of intersection distances"
      },
      {
        "name": "Hits",
        "nickname": "H",
        "access": "list",
        "description": "List of ray|obstacle hit flags"
      }
    ]
  },
  {
    "category": "Intersect",
    "subcategory": "Mathematical",
    "name": "Surface | Line",
    "nickname": "SCX",
    "guid": "c2c73357-bfd2-45af-89ff-40ca02a3442f",
    "description": "Solve intersection events for a surface and a line.",
    "inputs": [
      {
        "name": "Surface",
        "nickname": "S",
        "access": "item",
        "description": "Base surface"
      },
      {
        "name": "Line",
        "nickname": "L",
        "access": "item",
        "description": "Intersection line"
      },
      {
        "name": "First",
        "nickname": "F",
        "access": "item",
        "description": "Limit to first intersection"
      }
    ],
    "outputs": [
      {
        "name": "Curves",
        "nickname": "C",
        "access": "list",
        "description": "Intersection overlap curves"
      },
      {
        "name": "Points",
        "nickname": "P",
        "access": "list",
        "description": "Intersection points"
      },
      {
        "name": "UV Points",
        "nickname": "uv",
        "access": "list",
        "description": "Surface {uv} coordinates at intersection events"
      },
      {
        "name": "Normal",
        "nickname": "N",
        "access": "list",
        "description": "Surface normal vector at intersection events"
      }
    ]
  },
  {
    "category": "Intersect",
    "subcategory": "Mathematical",
    "name": "IsoVist",
    "nickname": "IVist",
    "guid": "cab92254-1c79-4e5a-9972-0a4412b35c88",
    "description": "Compute an isovist sampling at a location",
    "inputs": [
      {
        "name": "Plane",
        "nickname": "P",
        "access": "item",
        "description": "Sampling plane and origin"
      },
      {
        "name": "Count",
        "nickname": "N",
        "access": "item",
        "description": "Sample count"
      },
      {
        "name": "Radius",
        "nickname": "R",
        "access": "item",
        "description": "Sample radius"
      },
      {
        "name": "Obstacles",
        "nickname": "O",
        "access": "list",
        "description": "Obstacle outlines"
      }
    ],
    "outputs": [
      {
        "name": "Points",
        "nickname": "P",
        "access": "list",
        "description": "Intersection points of the sample rays with the obstacles"
      },
      {
        "name": "Distance",
        "nickname": "D",
        "access": "list",
        "description": "List of intersection distances"
      },
      {
        "name": "Index",
        "nickname": "I",
        "access": "list",
        "description": "List of obstacle indices for each hit, or -1 if no obstacle was hit"
      }
    ]
  },
  {
    "category": "Intersect",
    "subcategory": "Mathematical",
    "name": "Curve | Line [OBSOLETE]",
    "nickname": "CLX",
    "guid": "ddaea1a9-d6bd-4a18-ac11-8a4993954a03",
    "description": "Solve intersection events for a curve and a line.",
    "inputs": [
      {
        "name": "Curve",
        "nickname": "C",
        "access": "item",
        "description": "Curve to intersect"
      },
      {
        "name": "Line",
        "nickname": "L",
        "access": "item",
        "description": "Line to intersect with"
      }
    ],
    "outputs": [
      {
        "name": "Points",
        "nickname": "P",
        "access": "item",
        "description": "Intersection events"
      },
      {
        "name": "Params",
        "nickname": "t",
        "access": "item",
        "description": "Parameters on curve"
      }
    ]
  },
  {
    "category": "Intersect",
    "subcategory": "Mathematical",
    "name": "Brep | Line",
    "nickname": "BLX",
    "guid": "ed0742f9-6647-4d95-9dfd-9ad17080ae9c",
    "description": "Solve intersection events for a Brep and a line.",
    "inputs": [
      {
        "name": "Brep",
        "nickname": "B",
        "access": "item",
        "description": "Base Brep"
      },
      {
        "name": "Line",
        "nickname": "L",
        "access": "item",
        "description": "Intersection line"
      }
    ],
    "outputs": [
      {
        "name": "Curves",
        "nickname": "C",
        "access": "list",
        "description": "Intersection overlap curves"
      },
      {
        "name": "Points",
        "nickname": "P",
        "access": "list",
        "description": "Intersection points"
      }
    ]
  },
  {
    "category": "Intersect",
    "subcategory": "Shape",
    "name": "Split Brep",
    "nickname": "Split",
    "guid": "ef6b26f4-f820-48d6-b0c5-85898ef8888b",
    "description": "Split one brep with another.",
    "inputs": [
      {
        "name": "Brep",
        "nickname": "B",
        "access": "item",
        "description": "Brep to split"
      },
      {
        "name": "Cutter",
        "nickname": "C",
        "access": "item",
        "description": "Cutting shape"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "R",
        "access": "list",
        "description": "Brep fragments"
      }
    ]
  },
  {
    "category": "Intersect",
    "subcategory": "Shape",
    "name": "Trim Solid",
    "nickname": "Trim",
    "guid": "f0b70e8e-7337-4ce4-a7bb-317fc971f918",
    "description": "Cut holes into a shape with a set of solid cutters.",
    "inputs": [
      {
        "name": "Shape",
        "nickname": "S",
        "access": "item",
        "description": "Shape to trim"
      },
      {
        "name": "Cutters",
        "nickname": "T",
        "access": "list",
        "description": "Trimming shapes"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "R",
        "access": "list",
        "description": "Shape with holes"
      }
    ]
  },
  {
    "category": "Intersect",
    "subcategory": "Mathematical",
    "name": "Plane | Plane | Plane",
    "nickname": "3PX",
    "guid": "f1ea5a4b-1a4f-4cf4-ad94-1ecfb9302b6e",
    "description": "Solve the intersection events of three planes.",
    "inputs": [
      {
        "name": "Plane A",
        "nickname": "A",
        "access": "item",
        "description": "First plane"
      },
      {
        "name": "Plane B",
        "nickname": "B",
        "access": "item",
        "description": "Second plane"
      },
      {
        "name": "Plane C",
        "nickname": "C",
        "access": "item",
        "description": "Third plane"
      }
    ],
    "outputs": [
      {
        "name": "Point",
        "nickname": "Pt",
        "access": "item",
        "description": "Intersection point"
      },
      {
        "name": "Line AB",
        "nickname": "AB",
        "access": "item",
        "description": "Intersection line between A and B"
      },
      {
        "name": "Line AC",
        "nickname": "AC",
        "access": "item",
        "description": "Intersection line between A and C"
      },
      {
        "name": "Line BC",
        "nickname": "BC",
        "access": "item",
        "description": "Intersection line between B and C"
      }
    ]
  },
  {
    "category": "Intersect",
    "subcategory": "Shape",
    "name": "Region Difference",
    "nickname": "RDiff",
    "guid": "f72c480b-7ee6-42ef-9821-c371e9203b44",
    "description": "Difference between two sets of planar closed curves (regions)",
    "inputs": [
      {
        "name": "Curves A",
        "nickname": "A",
        "access": "list",
        "description": "Curves to subtract from."
      },
      {
        "name": "Curves B",
        "nickname": "B",
        "access": "list",
        "description": "Curves to subtract."
      },
      {
        "name": "Plane",
        "nickname": "P",
        "access": "item",
        "description": "Optional plane for boolean solution"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "R",
        "access": "list",
        "description": "Result outlines of boolean difference (A - B)"
      }
    ]
  },
  {
    "category": "Intersect",
    "subcategory": "Shape",
    "name": "Solid Difference",
    "nickname": "SDiff",
    "guid": "fab11c30-2d9c-4d15-ab3c-2289f1ae5c21",
    "description": "Perform a solid difference on two Brep sets.",
    "inputs": [
      {
        "name": "Breps A",
        "nickname": "A",
        "access": "list",
        "description": "First Brep set"
      },
      {
        "name": "Breps B",
        "nickname": "B",
        "access": "list",
        "description": "Second Brep set"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "R",
        "access": "list",
        "description": "Difference result"
      }
    ]
  },
  {
    "category": "Intersect",
    "subcategory": "Mathematical",
    "name": "Brep | Line",
    "nickname": "BLX",
    "guid": "ff880808-6daf-4f6c-88c1-058120ad6ba9",
    "description": "Solve intersection events for a Brep and a line.",
    "inputs": [
      {
        "name": "Brep",
        "nickname": "B",
        "access": "item",
        "description": "Base Brep"
      },
      {
        "name": "Line",
        "nickname": "L",
        "access": "item",
        "description": "Intersection line"
      },
      {
        "name": "First",
        "nickname": "F",
        "access": "item",
        "description": "Limit to first intersection"
      }
    ],
    "outputs": [
      {
        "name": "Curves",
        "nickname": "C",
        "access": "list",
        "description": "Intersection overlap curves"
      },
      {
        "name": "Points",
        "nickname": "P",
        "access": "list",
        "description": "Intersection points"
      }
    ]
  }
];
