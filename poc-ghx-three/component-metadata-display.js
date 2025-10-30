export const DISPLAY_COMPONENTS = [
  {
    "category": "Display",
    "subcategory": "Preview",
    "name": "Cloud Display",
    "nickname": "Cloud",
    "guid": "059b72b0-9bb3-4542-a805-2dcd27493164",
    "description": "Draw a collection of points as a fuzzy cloud",
    "inputs": [
      {
        "name": "Points",
        "nickname": "P",
        "access": "list",
        "description": "Location for each blob"
      },
      {
        "name": "Colours",
        "nickname": "C",
        "access": "list",
        "description": "Colour for each blob"
      },
      {
        "name": "Size",
        "nickname": "S",
        "access": "item",
        "description": "Size for each blob"
      }
    ],
    "outputs": []
  },
  {
    "category": "Display",
    "subcategory": "Colour",
    "name": "Split AHSL",
    "nickname": "AHSL",
    "guid": "0a1331c8-c58d-4b3f-a886-47051532e35e",
    "description": "Split a colour into floating point {AHSL} channels",
    "inputs": [
      {
        "name": "Colour",
        "nickname": "C",
        "access": "item",
        "description": "Input colour"
      }
    ],
    "outputs": [
      {
        "name": "Alpha",
        "nickname": "A",
        "access": "item",
        "description": "Alpha channel"
      },
      {
        "name": "Hue",
        "nickname": "H",
        "access": "item",
        "description": "Hue"
      },
      {
        "name": "Saturation",
        "nickname": "S",
        "access": "item",
        "description": "Saturation"
      },
      {
        "name": "Luminance",
        "nickname": "L",
        "access": "item",
        "description": "Luminance"
      }
    ]
  },
  {
    "category": "Display",
    "subcategory": "Vector",
    "name": "Point Order",
    "nickname": "Order",
    "guid": "0ad9f1ab-2204-45bb-b282-474469e2fa7b",
    "description": "Displays the order of a list of points",
    "inputs": [
      {
        "name": "Points",
        "nickname": "P",
        "access": "list",
        "description": "Points to display"
      }
    ],
    "outputs": []
  },
  {
    "category": "Display",
    "subcategory": "Vector",
    "name": "Vector Display Ex",
    "nickname": "VDisEx",
    "guid": "11e95a7b-1e2c-4b66-bd95-fcad51f8662a",
    "description": "Preview vectors in the viewport",
    "inputs": [
      {
        "name": "Point",
        "nickname": "P",
        "access": "item",
        "description": "Start point of vector"
      },
      {
        "name": "Vector",
        "nickname": "V",
        "access": "item",
        "description": "Vector to display"
      },
      {
        "name": "Colour",
        "nickname": "C",
        "access": "item",
        "description": "Colour of vector"
      },
      {
        "name": "Width",
        "nickname": "W",
        "access": "item",
        "description": "Width of vector lines"
      }
    ],
    "outputs": []
  },
  {
    "category": "Display",
    "subcategory": "Colour",
    "name": "Colour CMYK",
    "nickname": "CMYK",
    "guid": "17af01a5-a846-4769-9478-de1df65a0afa",
    "description": "Create a colour from floating point {CMYK} channels.",
    "inputs": [
      {
        "name": "Cyan",
        "nickname": "C",
        "access": "item",
        "description": "Cyan channel (cyan is defined in the range {0.0 to 1.0})"
      },
      {
        "name": "Magenta",
        "nickname": "M",
        "access": "item",
        "description": "Magenta channel (magenta is defined in the range {0.0 to 1.0})"
      },
      {
        "name": "Yellow",
        "nickname": "Y",
        "access": "item",
        "description": "Yellow channel (yellow is defined in the range {0.0 to 1.0})"
      },
      {
        "name": "Key",
        "nickname": "K",
        "access": "item",
        "description": "Key channel (key is defined in the range {0.0 to 1.0})"
      }
    ],
    "outputs": [
      {
        "name": "Colour",
        "nickname": "C",
        "access": "item",
        "description": "Resulting colour"
      }
    ]
  },
  {
    "category": "Display",
    "subcategory": "Dimensions",
    "name": "Arc Dimension",
    "nickname": "ArcDim",
    "guid": "1bd97813-4fec-4453-9645-4ac920844f9d",
    "description": "Create an angle annotation based on an arc.",
    "inputs": [
      {
        "name": "Arc",
        "nickname": "A",
        "access": "item",
        "description": "Arc guide"
      },
      {
        "name": "Offset",
        "nickname": "O",
        "access": "item",
        "description": "Dimension offset"
      },
      {
        "name": "Text",
        "nickname": "T",
        "access": "item",
        "description": "Dimension text"
      },
      {
        "name": "Size",
        "nickname": "S",
        "access": "item",
        "description": "Dimension size"
      }
    ],
    "outputs": []
  },
  {
    "category": "Display",
    "subcategory": "Vector",
    "name": "Point List",
    "nickname": "Points",
    "guid": "1f18e802-4ab9-444f-bf3c-3e7e421a2acf",
    "description": "Displays the indices in lists of points",
    "inputs": [
      {
        "name": "Points",
        "nickname": "P",
        "access": "list",
        "description": "Points to display"
      },
      {
        "name": "Tags",
        "nickname": "T",
        "access": "item",
        "description": "Draw point index numbers"
      },
      {
        "name": "Lines",
        "nickname": "L",
        "access": "item",
        "description": "Draw connecting lines"
      },
      {
        "name": "Size",
        "nickname": "S",
        "access": "item",
        "description": "Optional Font size (in units)"
      }
    ],
    "outputs": []
  },
  {
    "category": "Display",
    "subcategory": "Vector",
    "name": "Vector Display",
    "nickname": "VDis",
    "guid": "2a3f7078-2e25-4dd4-96f7-0efb491bd61c",
    "description": "Preview vectors in the viewport",
    "inputs": [
      {
        "name": "Anchor",
        "nickname": "A",
        "access": "item",
        "description": "Anchor point for preview vector"
      },
      {
        "name": "Vector",
        "nickname": "V",
        "access": "item",
        "description": "Vector to preview"
      }
    ],
    "outputs": []
  },
  {
    "category": "Display",
    "subcategory": "Dimensions",
    "name": "Make2D Perspective View",
    "nickname": "M2D Perspective",
    "guid": "33359c6d-984e-42f3-a869-0c3364ab33b6",
    "description": "Define a perspective view for a Make2D solution",
    "inputs": [
      {
        "name": "Camera",
        "nickname": "C",
        "access": "item",
        "description": "Camera position"
      },
      {
        "name": "Frame",
        "nickname": "F",
        "access": "item",
        "description": "Projection framing."
      }
    ],
    "outputs": [
      {
        "name": "View",
        "nickname": "V",
        "access": "item",
        "description": "Parallel view"
      }
    ]
  },
  {
    "category": "Display",
    "subcategory": "Colour",
    "name": "Split ARGB",
    "nickname": "ARGB",
    "guid": "350f7d03-a48f-4121-bcee-328cfe1ed9ef",
    "description": "Split a colour into floating point {ARGB} channels.",
    "inputs": [
      {
        "name": "Colour",
        "nickname": "C",
        "access": "item",
        "description": "Input colour"
      }
    ],
    "outputs": [
      {
        "name": "Alpha",
        "nickname": "A",
        "access": "item",
        "description": "Alpha channel"
      },
      {
        "name": "Red",
        "nickname": "R",
        "access": "item",
        "description": "Red channel"
      },
      {
        "name": "Green",
        "nickname": "G",
        "access": "item",
        "description": "Green channel"
      },
      {
        "name": "Blue",
        "nickname": "B",
        "access": "item",
        "description": "Blue channel"
      }
    ]
  },
  {
    "category": "Display",
    "subcategory": "Dimensions",
    "name": "Text Tag",
    "nickname": "Tag",
    "guid": "3b220754-4114-4170-b6c3-b286b86ed524",
    "description": "Represents a list of text tags in a Rhino viewport",
    "inputs": [
      {
        "name": "Location",
        "nickname": "L",
        "access": "item",
        "description": "Location of text tag"
      },
      {
        "name": "Text",
        "nickname": "T",
        "access": "item",
        "description": "The text to display"
      },
      {
        "name": "Colour",
        "nickname": "C",
        "access": "item",
        "description": "Optional colour for tag"
      }
    ],
    "outputs": []
  },
  {
    "category": "Display",
    "subcategory": "Dimensions",
    "name": "Aligned Dimension",
    "nickname": "AlignDim",
    "guid": "3de3d3a0-1a1b-488c-b3d9-3fba0fdf07a8",
    "description": "Create a distance annotation between two points",
    "inputs": [
      {
        "name": "Plane",
        "nickname": "P",
        "access": "item",
        "description": "Plane for dimension"
      },
      {
        "name": "Point A",
        "nickname": "A",
        "access": "item",
        "description": "First dimension point"
      },
      {
        "name": "Point B",
        "nickname": "B",
        "access": "item",
        "description": "Second dimension point"
      },
      {
        "name": "Offset",
        "nickname": "O",
        "access": "item",
        "description": "Offset for base line"
      },
      {
        "name": "Text",
        "nickname": "T",
        "access": "item",
        "description": "Dimension text"
      },
      {
        "name": "Size",
        "nickname": "S",
        "access": "item",
        "description": "Dimension size"
      }
    ],
    "outputs": []
  },
  {
    "category": "Display",
    "subcategory": "Dimensions",
    "name": "Make2D Parallel View",
    "nickname": "M2D Parallel",
    "guid": "3fc08088-d75d-43bc-83cc-7a654f156cb7",
    "description": "Define a parallel view for a Make2D solution",
    "inputs": [
      {
        "name": "Projection",
        "nickname": "P",
        "access": "item",
        "description": "View projection."
      }
    ],
    "outputs": [
      {
        "name": "View",
        "nickname": "V",
        "access": "item",
        "description": "Parallel view"
      }
    ]
  },
  {
    "category": "Display",
    "subcategory": "Colour",
    "name": "Colour RGB",
    "nickname": "RGB",
    "guid": "49d2e200-b34e-4e1c-82a3-07feb4cb9378",
    "description": "Create a colour from {RGB} channels.",
    "inputs": [
      {
        "name": "Alpha",
        "nickname": "A",
        "access": "item",
        "description": "Alpha channel (255 = opaque)"
      },
      {
        "name": "Red",
        "nickname": "R",
        "access": "item",
        "description": "Red channel"
      },
      {
        "name": "Green",
        "nickname": "G",
        "access": "item",
        "description": "Green channel"
      },
      {
        "name": "Blue",
        "nickname": "B",
        "access": "item",
        "description": "Blue channel"
      }
    ],
    "outputs": [
      {
        "name": "Colour",
        "nickname": "C",
        "access": "item",
        "description": "Resulting colour"
      }
    ]
  },
  {
    "category": "Display",
    "subcategory": "Dimensions",
    "name": "Make2D Rhino View",
    "nickname": "M2D Rhino",
    "guid": "4ac24770-e38b-4363-be38-551a3b134707",
    "description": "Import a Rhino view for a Make2D solution",
    "inputs": [
      {
        "name": "Name",
        "nickname": "N",
        "access": "item",
        "description": "Named view or viewport name."
      },
      {
        "name": "Clip",
        "nickname": "C",
        "access": "item",
        "description": "If true, the view will be clipped to the frustum."
      }
    ],
    "outputs": [
      {
        "name": "View",
        "nickname": "V",
        "access": "item",
        "description": "Parallel view"
      }
    ]
  },
  {
    "category": "Display",
    "subcategory": "Dimensions",
    "name": "Linear Dimension",
    "nickname": "LinearDim",
    "guid": "5018bf8d-8566-4917-a6e3-5a623bda8079",
    "description": "Create a distance annotation between points, projected to a line.",
    "inputs": [
      {
        "name": "Line",
        "nickname": "L",
        "access": "item",
        "description": "Dimension base line"
      },
      {
        "name": "Point A",
        "nickname": "A",
        "access": "item",
        "description": "First dimension point"
      },
      {
        "name": "Point B",
        "nickname": "B",
        "access": "item",
        "description": "Second dimension point"
      },
      {
        "name": "Text",
        "nickname": "T",
        "access": "item",
        "description": "Dimension text"
      },
      {
        "name": "Size",
        "nickname": "S",
        "access": "item",
        "description": "Dimension size"
      }
    ],
    "outputs": []
  },
  {
    "category": "Display",
    "subcategory": "Preview",
    "name": "Custom Preview",
    "nickname": "Preview",
    "guid": "537b0419-bbc2-4ff4-bf08-afe526367b2c",
    "description": "Allows for customized geometry previews",
    "inputs": [
      {
        "name": "Geometry",
        "nickname": "G",
        "access": "item",
        "description": "Geometry to preview"
      },
      {
        "name": "Material",
        "nickname": "M",
        "access": "item",
        "description": "The material override"
      }
    ],
    "outputs": []
  },
  {
    "category": "Display",
    "subcategory": "Colour",
    "name": "Colour HSV",
    "nickname": "HSV",
    "guid": "5958a658-20c2-4a2b-86ba-4d1b81bf5348",
    "description": "Create a colour from floating point {HSV} channels.",
    "inputs": [
      {
        "name": "Alpha",
        "nickname": "A",
        "access": "item",
        "description": "Alpha channel (alpha is defined in the range {0.0 to 1.0})"
      },
      {
        "name": "Hue",
        "nickname": "H",
        "access": "item",
        "description": "Hue channel (hue is defined in the range {0.0 to 1.0})"
      },
      {
        "name": "Saturation",
        "nickname": "S",
        "access": "item",
        "description": "Saturation channel (saturation is defined in the range {0.0 to 1.0})"
      },
      {
        "name": "Value",
        "nickname": "V",
        "access": "item",
        "description": "Value channel (value/brightness is defined in the range {0.0 to 1.0})"
      }
    ],
    "outputs": [
      {
        "name": "Colour",
        "nickname": "C",
        "access": "item",
        "description": "Resulting colour"
      }
    ]
  },
  {
    "category": "Display",
    "subcategory": "Dimensions",
    "name": "Text Tag 3D",
    "nickname": "Tag",
    "guid": "5a41528b-12b9-40dc-a3f2-842034d267c4",
    "description": "Represents a list of 3D text tags in a Rhino viewport",
    "inputs": [
      {
        "name": "Location",
        "nickname": "L",
        "access": "item",
        "description": "Location and orientation of text tag"
      },
      {
        "name": "Text",
        "nickname": "T",
        "access": "item",
        "description": "The text to display"
      },
      {
        "name": "Size",
        "nickname": "S",
        "access": "item",
        "description": "Size of text"
      },
      {
        "name": "Colour",
        "nickname": "C",
        "access": "item",
        "description": "Optional colour of tag"
      },
      {
        "name": "Justification",
        "nickname": "J",
        "access": "item",
        "description": "Text justification"
      }
    ],
    "outputs": []
  },
  {
    "category": "Display",
    "subcategory": "Dimensions",
    "name": "Pattern Hatch",
    "nickname": "PHatch",
    "guid": "5f9e4549-8135-4a90-97c8-8a34bf05e99a",
    "description": "Create a patterned hatch",
    "inputs": [
      {
        "name": "Boundaries",
        "nickname": "B",
        "access": "list",
        "description": "Boundary curves for hatch objects"
      },
      {
        "name": "Pattern",
        "nickname": "P",
        "access": "item",
        "description": "Hatch pattern style"
      },
      {
        "name": "Scale",
        "nickname": "S",
        "access": "item",
        "description": "Pattern scale"
      },
      {
        "name": "Angle",
        "nickname": "A",
        "access": "item",
        "description": "Pattern angle"
      }
    ],
    "outputs": []
  },
  {
    "category": "Display",
    "subcategory": "Colour",
    "name": "Addition",
    "nickname": "Add",
    "guid": "60327ca4-c548-40e6-a11f-3c6759582f13",
    "description": "Perform colour addition.",
    "inputs": [
      {
        "name": "Colour A",
        "nickname": "A",
        "access": "item",
        "description": "First colour"
      },
      {
        "name": "Colour B",
        "nickname": "B",
        "access": "item",
        "description": "Second colour"
      }
    ],
    "outputs": [
      {
        "name": "Colour",
        "nickname": "C",
        "access": "item",
        "description": "Resulting colour"
      }
    ]
  },
  {
    "category": "Display",
    "subcategory": "Preview",
    "name": "Symbol Display",
    "nickname": "Symbol",
    "guid": "62d5ead4-53c4-4d0b-b5ce-6bd6e0850ab8",
    "description": "Display symbols",
    "inputs": [
      {
        "name": "Location",
        "nickname": "P",
        "access": "item",
        "description": "Symbol location"
      },
      {
        "name": "Display",
        "nickname": "D",
        "access": "item",
        "description": "Symbol display properties"
      }
    ],
    "outputs": []
  },
  {
    "category": "Display",
    "subcategory": "Preview",
    "name": "Dot Display",
    "nickname": "Dots",
    "guid": "6b1bd8b2-47a4-4aa6-a471-3fd91c62a486",
    "description": "Draw a collection of coloured dots",
    "inputs": [
      {
        "name": "Point",
        "nickname": "P",
        "access": "item",
        "description": "Dot location"
      },
      {
        "name": "Colour",
        "nickname": "C",
        "access": "item",
        "description": "Dot colour"
      },
      {
        "name": "Size",
        "nickname": "S",
        "access": "item",
        "description": "Dot size"
      }
    ],
    "outputs": []
  },
  {
    "category": "Display",
    "subcategory": "Dimensions",
    "name": "Gradient Hatch",
    "nickname": "GHatch",
    "guid": "6ce90407-9eac-4a1a-a81a-949b601f18f3",
    "description": "Create a gradient hatch",
    "inputs": [
      {
        "name": "Boundaries",
        "nickname": "B",
        "access": "list",
        "description": "Boundary curves for hatch objects"
      },
      {
        "name": "Axis",
        "nickname": "A",
        "access": "item",
        "description": "Gradient axis"
      },
      {
        "name": "Colour 1",
        "nickname": "C1",
        "access": "item",
        "description": "Colour at start of axis."
      },
      {
        "name": "Colour 2",
        "nickname": "C2",
        "access": "item",
        "description": "Colour at end of axis."
      }
    ],
    "outputs": []
  },
  {
    "category": "Display",
    "subcategory": "Colour",
    "name": "Colour LCH",
    "nickname": "LCH",
    "guid": "75a07554-8a2c-4d87-81b9-d854f498509d",
    "description": "Create a colour from floating point {CIE LCH} channels.",
    "inputs": [
      {
        "name": "Alpha",
        "nickname": "A",
        "access": "item",
        "description": "Alpha channel (alpha is defined in the range {0.0 to 1.0})"
      },
      {
        "name": "Luminance",
        "nickname": "L",
        "access": "item",
        "description": "Luminance channel (luminance is defined in the range {0.0 to 1.0})"
      },
      {
        "name": "Chroma",
        "nickname": "C",
        "access": "item",
        "description": "Chromaticity channel (chroma is defined in the range {0.0 to 1.0})"
      },
      {
        "name": "Hue",
        "nickname": "H",
        "access": "item",
        "description": "Hue channel (hue is defined in the range {0.0 to 1.0})"
      }
    ],
    "outputs": [
      {
        "name": "Colour",
        "nickname": "C",
        "access": "item",
        "description": "Resulting colour"
      }
    ]
  },
  {
    "category": "Display",
    "subcategory": "Preview",
    "name": "Create Material",
    "nickname": "Material",
    "guid": "76975309-75a6-446a-afed-f8653720a9f2",
    "description": "Create an OpenGL material.",
    "inputs": [
      {
        "name": "Diffuse",
        "nickname": "Kd",
        "access": "item",
        "description": "Colour of the diffuse channel"
      },
      {
        "name": "Specular",
        "nickname": "Ks",
        "access": "item",
        "description": "Colour of the specular highlight"
      },
      {
        "name": "Emission",
        "nickname": "Ke",
        "access": "item",
        "description": "Emissive colour of the material"
      },
      {
        "name": "Transparency",
        "nickname": "T",
        "access": "item",
        "description": "Amount of transparency (0.0 = opaque, 1.0 = transparent"
      },
      {
        "name": "Shine",
        "nickname": "S",
        "access": "item",
        "description": "Amount of shinyness (0 = none, 1 = low shine, 100 = max shine"
      }
    ],
    "outputs": [
      {
        "name": "Material",
        "nickname": "M",
        "access": "item",
        "description": "Resulting material"
      }
    ]
  },
  {
    "category": "Display",
    "subcategory": "Colour",
    "name": "Colour XYZ",
    "nickname": "XYZ",
    "guid": "77185dc2-2f18-469d-9686-00f5b6049195",
    "description": "Create a colour from floating point {XYZ} channels (CIE 1931 spec).",
    "inputs": [
      {
        "name": "Alpha",
        "nickname": "A",
        "access": "item",
        "description": "Alpha channel (alpha is defined in the range {0.0 to 1.0})"
      },
      {
        "name": "X",
        "nickname": "X",
        "access": "item",
        "description": "X stimulus (X is defined in the range {0.0 to 1.0})"
      },
      {
        "name": "Y",
        "nickname": "Y",
        "access": "item",
        "description": "Y stimulus (y is defined in the range {0.0 to 1.0})"
      },
      {
        "name": "Z",
        "nickname": "Z",
        "access": "item",
        "description": "Z stimulus (Z is defined in the range {0.0 to 1.0})"
      }
    ],
    "outputs": [
      {
        "name": "Colour",
        "nickname": "C",
        "access": "item",
        "description": "Resulting colour"
      }
    ]
  },
  {
    "category": "Display",
    "subcategory": "Preview",
    "name": "Symbol (Simple)",
    "nickname": "SymSim",
    "guid": "79747717-1874-4c34-b790-faef53b50569",
    "description": "Simple symbol display properties",
    "inputs": [
      {
        "name": "Style",
        "nickname": "X",
        "access": "item",
        "description": "Symbol style"
      },
      {
        "name": "Size",
        "nickname": "S",
        "access": "item",
        "description": "Primary radius or outer size"
      },
      {
        "name": "Rotation",
        "nickname": "R",
        "access": "item",
        "description": "Rotation angle"
      },
      {
        "name": "Colour",
        "nickname": "C",
        "access": "item",
        "description": "Main colour"
      }
    ],
    "outputs": [
      {
        "name": "Symbol Display",
        "nickname": "D",
        "access": "item",
        "description": "Symbol display properties"
      }
    ]
  },
  {
    "category": "Display",
    "subcategory": "Dimensions",
    "name": "Serial Dimension",
    "nickname": "SerialDim",
    "guid": "7dd42002-75bb-4f41-857f-472a140b3b28",
    "description": "Create a distance annotation between multiple points, projected to a line.",
    "inputs": [
      {
        "name": "Line",
        "nickname": "L",
        "access": "item",
        "description": "Dimension base line"
      },
      {
        "name": "Points",
        "nickname": "P",
        "access": "list",
        "description": "Dimension points, the first one marks the zero point"
      },
      {
        "name": "Text",
        "nickname": "T",
        "access": "item",
        "description": "Dimension text"
      },
      {
        "name": "Size",
        "nickname": "S",
        "access": "item",
        "description": "Dimension size"
      }
    ],
    "outputs": []
  },
  {
    "category": "Display",
    "subcategory": "Dimensions",
    "name": "Circular Dimension",
    "nickname": "CircleDim",
    "guid": "7e9489e0-122d-401a-aba8-f1dae0217c40",
    "description": "Create an angle annotation projected to a circle.",
    "inputs": [
      {
        "name": "Circle",
        "nickname": "C",
        "access": "item",
        "description": "Dimension guide circle"
      },
      {
        "name": "Point A",
        "nickname": "A",
        "access": "item",
        "description": "First angle point"
      },
      {
        "name": "Point B",
        "nickname": "B",
        "access": "item",
        "description": "Second angle point"
      },
      {
        "name": "Text",
        "nickname": "T",
        "access": "item",
        "description": "Dimension text"
      },
      {
        "name": "Size",
        "nickname": "S",
        "access": "item",
        "description": "Dimension size"
      }
    ],
    "outputs": []
  },
  {
    "category": "Display",
    "subcategory": "Dimensions",
    "name": "Angular Dimensions (Mesh)",
    "nickname": "AngleDimMesh",
    "guid": "91f3bde5-26e6-432e-a5fe-a2938b2a94f9",
    "description": "Create angle annotations for all mesh corners.",
    "inputs": [
      {
        "name": "Mesh",
        "nickname": "M",
        "access": "item",
        "description": "Mesh to annotate"
      },
      {
        "name": "Text",
        "nickname": "T",
        "access": "item",
        "description": "Dimension text"
      },
      {
        "name": "Size",
        "nickname": "S",
        "access": "item",
        "description": "Dimension size"
      },
      {
        "name": "Length Factor",
        "nickname": "F",
        "access": "item",
        "description": "Radius of dimension as part of edge length."
      },
      {
        "name": "Minimum Angle",
        "nickname": "A0",
        "access": "item",
        "description": "Threshold angle below which dimensions are not drawn."
      },
      {
        "name": "Maximum Angle",
        "nickname": "A1",
        "access": "item",
        "description": "Threshold angle above which dimensions are not drawn."
      }
    ],
    "outputs": []
  },
  {
    "category": "Display",
    "subcategory": "Dimensions",
    "name": "Make2D",
    "nickname": "Make2D",
    "guid": "96e40f6b-ba46-4102-bf15-ebf90471f4a0",
    "description": "Create a hidden line drawing from geometry",
    "inputs": [
      {
        "name": "Geometry",
        "nickname": "G",
        "access": "list",
        "description": "Geometry to include (Breps, Meshes and Curves only)."
      },
      {
        "name": "Clipping Planes",
        "nickname": "C",
        "access": "list",
        "description": "Optional clipping planes."
      },
      {
        "name": "View",
        "nickname": "V",
        "access": "item",
        "description": "Make2D projection details"
      },
      {
        "name": "Tangent Edges",
        "nickname": "Te",
        "access": "item",
        "description": "Whether or not to compute tangent edges."
      },
      {
        "name": "Tangent Seams",
        "nickname": "Ts",
        "access": "item",
        "description": "Whether or not to compute tangent seams."
      }
    ],
    "outputs": [
      {
        "name": "Visible curves",
        "nickname": "V",
        "access": "list",
        "description": "List of visible curves"
      },
      {
        "name": "Visible index",
        "nickname": "Vi",
        "access": "list",
        "description": "For each visible curve, index of source object"
      },
      {
        "name": "Visible type",
        "nickname": "Vt",
        "access": "list",
        "description": "For each visible curve, type description"
      },
      {
        "name": "Hidden curves",
        "nickname": "H",
        "access": "list",
        "description": "List of hidden curves"
      },
      {
        "name": "Hidden index",
        "nickname": "Hi",
        "access": "list",
        "description": "For each hidden curve, index of source object"
      },
      {
        "name": "Hidden type",
        "nickname": "Ht",
        "access": "list",
        "description": "For each hidden curve, type description"
      }
    ]
  },
  {
    "category": "Display",
    "subcategory": "Colour",
    "name": "Colour HSL",
    "nickname": "HSL",
    "guid": "a45d68b3-c299-4b17-bdae-7975f216cec6",
    "description": "Create a colour from floating point {HSL} channels.",
    "inputs": [
      {
        "name": "Alpha",
        "nickname": "A",
        "access": "item",
        "description": "Alpha channel (alpha is defined in the range {0.0 to 1.0})"
      },
      {
        "name": "Hue",
        "nickname": "H",
        "access": "item",
        "description": "Hue channel (hue is defined in the range {0.0 to 1.0})"
      },
      {
        "name": "Saturation",
        "nickname": "S",
        "access": "item",
        "description": "Saturation channel (saturation is defined in the range {0.0 to 1.0})"
      },
      {
        "name": "Luminance",
        "nickname": "L",
        "access": "item",
        "description": "Luminance channel (luminance is defined in the range {0.0 to 1.0})"
      }
    ],
    "outputs": [
      {
        "name": "Colour",
        "nickname": "C",
        "access": "item",
        "description": "Resulting colour"
      }
    ]
  },
  {
    "category": "Display",
    "subcategory": "Viewport",
    "name": "Viewport Display",
    "nickname": "Viewport Display",
    "guid": "b78d95bc-dffb-414c-b177-c611c92580b9",
    "description": "Display viewport on canvas",
    "inputs": [
      {
        "name": "Visible",
        "nickname": "V",
        "access": "item",
        "description": "Show viewport"
      },
      {
        "name": "Left",
        "nickname": "L",
        "access": "item",
        "description": "Viewport left"
      },
      {
        "name": "Top",
        "nickname": "T",
        "access": "item",
        "description": "Viewport top"
      },
      {
        "name": "Width",
        "nickname": "W",
        "access": "item",
        "description": "Viewport width"
      },
      {
        "name": "Height",
        "nickname": "H",
        "access": "item",
        "description": "Viewport height"
      }
    ],
    "outputs": []
  },
  {
    "category": "Display",
    "subcategory": "Dimensions",
    "name": "Marker Dimension",
    "nickname": "MarkDim",
    "guid": "c5208969-16f9-48af-8a86-e500c033fb76",
    "description": "Create a text annotation at a point",
    "inputs": [
      {
        "name": "Line",
        "nickname": "L",
        "access": "item",
        "description": "Dimension base line"
      },
      {
        "name": "Text",
        "nickname": "T",
        "access": "item",
        "description": "Dimension text"
      },
      {
        "name": "Size",
        "nickname": "S",
        "access": "item",
        "description": "Dimension size"
      }
    ],
    "outputs": []
  },
  {
    "category": "Display",
    "subcategory": "Vector",
    "name": "Point List",
    "nickname": "Points",
    "guid": "cc14daa5-911a-4fcc-8b3b-1149bf7f2eeb",
    "description": "Displays details about lists of points",
    "inputs": [
      {
        "name": "Points",
        "nickname": "P",
        "access": "list",
        "description": "Points to display"
      },
      {
        "name": "Size",
        "nickname": "S",
        "access": "item",
        "description": "Optional text size (in Rhino units)"
      }
    ],
    "outputs": []
  },
  {
    "category": "Display",
    "subcategory": "Dimensions",
    "name": "Line Dimension",
    "nickname": "LineDim",
    "guid": "d78f026a-0109-4bcc-bf91-d08475711466",
    "description": "Create a distance annotation along a line.",
    "inputs": [
      {
        "name": "Line",
        "nickname": "L",
        "access": "item",
        "description": "Dimension base line"
      },
      {
        "name": "Text",
        "nickname": "T",
        "access": "item",
        "description": "Dimension text"
      },
      {
        "name": "Size",
        "nickname": "S",
        "access": "item",
        "description": "Dimension size"
      }
    ],
    "outputs": []
  },
  {
    "category": "Display",
    "subcategory": "Colour",
    "name": "Split AHSV",
    "nickname": "AHSV",
    "guid": "d84d2c2a-2813-4667-afb4-46642581e5f9",
    "description": "Split a colour into floating point {AHSV} channels",
    "inputs": [
      {
        "name": "Colour",
        "nickname": "C",
        "access": "item",
        "description": "Input colour"
      }
    ],
    "outputs": [
      {
        "name": "Alpha",
        "nickname": "A",
        "access": "item",
        "description": "Alpha channel"
      },
      {
        "name": "Hue",
        "nickname": "H",
        "access": "item",
        "description": "Hue"
      },
      {
        "name": "Saturation",
        "nickname": "S",
        "access": "item",
        "description": "Saturation"
      },
      {
        "name": "Value",
        "nickname": "V",
        "access": "item",
        "description": "Value (Brightness)"
      }
    ]
  },
  {
    "category": "Display",
    "subcategory": "Preview",
    "name": "Symbol (Advanced)",
    "nickname": "SymAdv",
    "guid": "e5c82975-8011-412c-b56d-bb7fc9e7f28d",
    "description": "Advanced symbol display properties",
    "inputs": [
      {
        "name": "Style",
        "nickname": "X",
        "access": "item",
        "description": "Symbol style"
      },
      {
        "name": "Size Primary",
        "nickname": "S1",
        "access": "item",
        "description": "Symbol size"
      },
      {
        "name": "Size Secondary",
        "nickname": "S2",
        "access": "item",
        "description": "Alternative size or offset (depending on style)."
      },
      {
        "name": "Rotation",
        "nickname": "R",
        "access": "item",
        "description": "Rotation angle"
      },
      {
        "name": "Fill",
        "nickname": "Cf",
        "access": "item",
        "description": "Fill colour"
      },
      {
        "name": "Edge",
        "nickname": "Ce",
        "access": "item",
        "description": "Edge colour"
      },
      {
        "name": "Width",
        "nickname": "W",
        "access": "item",
        "description": "Edge width"
      },
      {
        "name": "Adjust",
        "nickname": "A",
        "access": "item",
        "description": "Adjust apparent size based on view"
      }
    ],
    "outputs": [
      {
        "name": "Symbol Display",
        "nickname": "D",
        "access": "item",
        "description": "Symbol display properties"
      }
    ]
  },
  {
    "category": "Display",
    "subcategory": "Colour",
    "name": "Colour RGB (f)",
    "nickname": "fRGB",
    "guid": "f35132c0-c298-4b9c-b446-42e960f52677",
    "description": "Create a colour from floating point {RGB} channels.",
    "inputs": [
      {
        "name": "Alpha",
        "nickname": "A",
        "access": "item",
        "description": "Alpha channel (1.0 = opaque)"
      },
      {
        "name": "Red",
        "nickname": "R",
        "access": "item",
        "description": "Red channel"
      },
      {
        "name": "Green",
        "nickname": "G",
        "access": "item",
        "description": "Green channel"
      },
      {
        "name": "Blue",
        "nickname": "B",
        "access": "item",
        "description": "Blue channel"
      }
    ],
    "outputs": [
      {
        "name": "Colour",
        "nickname": "C",
        "access": "item",
        "description": "Resulting colour"
      }
    ]
  },
  {
    "category": "Display",
    "subcategory": "Test",
    "name": "Test Crash",
    "nickname": "Test Crash",
    "guid": "f3c769fd-aa9b-4695-a1ce-3ad4c53d1440",
    "description": "Test crashing of GH",
    "inputs": [
      {
        "name": "Crash",
        "nickname": "C",
        "access": "item",
        "description": "crash"
      }
    ],
    "outputs": []
  },
  {
    "category": "Display",
    "subcategory": "Graphs",
    "name": "Legend",
    "nickname": "Legend",
    "guid": "f6867cdd-2216-4451-9134-7da94bdcd5af",
    "description": "Display a legend consisting of Tags and Colours",
    "inputs": [
      {
        "name": "Colour",
        "nickname": "C",
        "access": "list",
        "description": "Legend colours"
      },
      {
        "name": "Tags",
        "nickname": "T",
        "access": "list",
        "description": "Legend tags"
      },
      {
        "name": "Rectangle",
        "nickname": "R",
        "access": "item",
        "description": "Optional legend rectangle in 3D space"
      }
    ],
    "outputs": []
  },
  {
    "category": "Display",
    "subcategory": "Colour",
    "name": "Colour L*ab",
    "nickname": "L*AB",
    "guid": "f922ed44-6e4a-44a0-8b4b-4b4a46bdfe29",
    "description": "Create a colour from floating point {CIE L*ab} channels.",
    "inputs": [
      {
        "name": "Alpha",
        "nickname": "A",
        "access": "item",
        "description": "Alpha channel (alpha is defined in the range {0.0 to 1.0})"
      },
      {
        "name": "Luminance",
        "nickname": "L",
        "access": "item",
        "description": "Luminance channel (luminance is defined in the range {0.0 to 1.0})"
      },
      {
        "name": "A",
        "nickname": "A",
        "access": "item",
        "description": "First colour channel (A is defined in the range {0.0 to 1.0})"
      },
      {
        "name": "B",
        "nickname": "B",
        "access": "item",
        "description": "Opposing colour channel (B is defined in the range {0.0 to 1.0})"
      }
    ],
    "outputs": [
      {
        "name": "Colour",
        "nickname": "C",
        "access": "item",
        "description": "Resulting colour"
      }
    ]
  },
  {
    "category": "Display",
    "subcategory": "Dimensions",
    "name": "Angular Dimension",
    "nickname": "AngleDim",
    "guid": "fc6b519e-df6d-4ce1-a1f4-083f1c217c14",
    "description": "Create an angle annotation between points.",
    "inputs": [
      {
        "name": "Center",
        "nickname": "C",
        "access": "item",
        "description": "Angle centre point"
      },
      {
        "name": "Point A",
        "nickname": "A",
        "access": "item",
        "description": "End of first angle direction"
      },
      {
        "name": "Point B",
        "nickname": "B",
        "access": "item",
        "description": "End of second angle direction"
      },
      {
        "name": "Reflex",
        "nickname": "R",
        "access": "item",
        "description": "Create dimension for reflex angle"
      },
      {
        "name": "Text",
        "nickname": "T",
        "access": "item",
        "description": "Dimension text"
      },
      {
        "name": "Size",
        "nickname": "S",
        "access": "item",
        "description": "Dimension size"
      }
    ],
    "outputs": []
  }
];
