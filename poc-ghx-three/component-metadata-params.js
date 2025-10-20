export const PARAMS_COMPONENTS = [
  {
    "category": "Params",
    "subcategory": "Input",
    "name": "Import 3DM",
    "nickname": "3DM",
    "guid": "317f1cb2-820d-4a8f-b5c8-5de3594ddfba",
    "description": "Import geometry into a RhinoDoc",
    "inputs": [
      {
        "name": "File",
        "nickname": "F",
        "access": "item",
        "description": "Location of file"
      },
      {
        "name": "Layer",
        "nickname": "L",
        "access": "item",
        "description": "Layer name filter"
      },
      {
        "name": "Name",
        "nickname": "N",
        "access": "item",
        "description": "Object name filter"
      }
    ],
    "outputs": [
      {
        "name": "Geometry",
        "nickname": "G",
        "access": "list",
        "description": "Imported geometry"
      }
    ]
  },
  {
    "category": "Params",
    "subcategory": "Special",
    "name": "Timeline",
    "nickname": "Timeline",
    "guid": "33eb59b9-5f81-4ef5-8c89-46e6e744522b",
    "description": "A timeline of values",
    "inputs": [
      {
        "name": "A",
        "nickname": "A",
        "access": "item",
        "description": "Value domain for parameter"
      }
    ],
    "outputs": [
      {
        "name": "Value A",
        "nickname": "A",
        "access": "item",
        "description": "Interpolated values for A"
      }
    ]
  },
  {
    "category": "Params",
    "subcategory": "Input",
    "name": "Import PDB",
    "nickname": "PDB",
    "guid": "383929c0-6515-4899-8b4b-3bd0d0b32471",
    "description": "Import data from Protein Data Bank *.pdb files.",
    "inputs": [
      {
        "name": "File",
        "nickname": "F",
        "access": "item",
        "description": "Location of *.pdb file"
      }
    ],
    "outputs": [
      {
        "name": "Atoms",
        "nickname": "A",
        "access": "list",
        "description": "All atoms in the PDB file"
      },
      {
        "name": "Bonds",
        "nickname": "B",
        "access": "list",
        "description": "Bonds between atoms"
      }
    ]
  },
  {
    "category": "Params",
    "subcategory": "Util",
    "name": "Data Dam",
    "nickname": "Dam",
    "guid": "65283518-ad00-49d3-87fb-f76823ebb162",
    "description": "Delay data on its way through the document",
    "inputs": [
      {
        "name": "Data A",
        "nickname": "A",
        "access": "tree",
        "description": "Data to buffer"
      }
    ],
    "outputs": [
      {
        "name": "Data A",
        "nickname": "A",
        "access": "tree",
        "description": "Buffered data"
      }
    ]
  },
  {
    "category": "Params",
    "subcategory": "Input",
    "name": "Read File",
    "nickname": "File",
    "guid": "6587fcbf-e3cf-480a-b2f5-641794474194",
    "description": "Read the contents of a file",
    "inputs": [
      {
        "name": "File",
        "nickname": "F",
        "access": "item",
        "description": "Uri of file to read"
      }
    ],
    "outputs": [
      {
        "name": "Content",
        "nickname": "C",
        "access": "list",
        "description": "File content"
      }
    ]
  },
  {
    "category": "Params",
    "subcategory": "Input",
    "name": "Gradient",
    "nickname": "Gradient",
    "guid": "6da9f120-3ad0-4b6e-9fe0-f8cde3a649b7",
    "description": "Represents a multiple colour gradient",
    "inputs": [
      {
        "name": "Lower limit",
        "nickname": "L0",
        "access": "item",
        "description": "Lower limit of gradient range"
      },
      {
        "name": "Upper limit",
        "nickname": "L1",
        "access": "item",
        "description": "Upper limit of gradient range"
      },
      {
        "name": "Parameter",
        "nickname": "t",
        "access": "item",
        "description": "Parameter along gradient range"
      }
    ],
    "outputs": [
      {
        "name": "Colour",
        "nickname": "C",
        "access": "item",
        "description": "Colour along gradient at parameter"
      }
    ]
  },
  {
    "category": "Params",
    "subcategory": "Util",
    "name": "Context Print",
    "nickname": "Context Print",
    "guid": "73215ec5-0eb5-4f85-9e07-b09c4590ce2b",
    "description": "Textual data to print at the end of the GrasshopperPlayer command.",
    "inputs": [
      {
        "name": "Text",
        "nickname": "Tx",
        "access": "tree",
        "description": "Text for printing."
      }
    ],
    "outputs": []
  },
  {
    "category": "Params",
    "subcategory": "Input",
    "name": "Atom Data",
    "nickname": "Atom",
    "guid": "7b371d04-53e3-47d8-b3dd-7b113c48bc59",
    "description": "Get detailed information for an atom",
    "inputs": [
      {
        "name": "Atom",
        "nickname": "A",
        "access": "item",
        "description": "Atom to evaluate"
      }
    ],
    "outputs": [
      {
        "name": "Point",
        "nickname": "P",
        "access": "item",
        "description": "Location of atom"
      },
      {
        "name": "Element",
        "nickname": "E",
        "access": "item",
        "description": "Element name of atom"
      },
      {
        "name": "Chain",
        "nickname": "C",
        "access": "item",
        "description": "Chain ID to which this atom belongs"
      },
      {
        "name": "Residue",
        "nickname": "R",
        "access": "item",
        "description": "Residue name to which this atom belongs"
      },
      {
        "name": "Charge",
        "nickname": "e",
        "access": "item",
        "description": "Charge of this atom"
      },
      {
        "name": "Occupancy",
        "nickname": "O",
        "access": "item",
        "description": "Occupancy of this atom"
      },
      {
        "name": "Temperature",
        "nickname": "T",
        "access": "item",
        "description": "Temperature factor of this atom"
      },
      {
        "name": "Atomic Number",
        "nickname": "AN",
        "access": "item",
        "description": "Atomic number of atom"
      },
      {
        "name": "Serial Number",
        "nickname": "SN",
        "access": "item",
        "description": "Atom serial number"
      },
      {
        "name": "Residue Number",
        "nickname": "RN",
        "access": "item",
        "description": "Residue serial number"
      }
    ]
  },
  {
    "category": "Params",
    "subcategory": "Util",
    "name": "Cluster",
    "nickname": "Cluster",
    "guid": "865c8275-d9db-4b9a-92d4-883ef3b00b4a",
    "description": "Contains a cluster of Grasshopper components",
    "inputs": [],
    "outputs": []
  },
  {
    "category": "Params",
    "subcategory": "Input",
    "name": "Import SHP",
    "nickname": "SHP",
    "guid": "aa538b89-3df8-436f-9ae4-bc44525984de",
    "description": "Import data from GIS *.shp files.",
    "inputs": [
      {
        "name": "File",
        "nickname": "F",
        "access": "item",
        "description": "Location of *.shp file"
      }
    ],
    "outputs": [
      {
        "name": "Points",
        "nickname": "P",
        "access": "list",
        "description": "Points in file"
      },
      {
        "name": "Curves",
        "nickname": "C",
        "access": "list",
        "description": "Curves in file"
      },
      {
        "name": "Regions",
        "nickname": "R",
        "access": "list",
        "description": "Regions in file"
      }
    ]
  },
  {
    "category": "Params",
    "subcategory": "Util",
    "name": "Context Bake",
    "nickname": "Context Bake",
    "guid": "ae2531b4-bab2-4bb1-b5bf-f2143d10c132",
    "description": "Geometry for baking at the end of the GrasshopperPlayer command.",
    "inputs": [
      {
        "name": "Geometry",
        "nickname": "Geometry",
        "access": "tree",
        "description": "Geometry to collect for baking"
      }
    ],
    "outputs": []
  },
  {
    "category": "Params",
    "subcategory": "Input",
    "name": "Import Coordinates",
    "nickname": "Coords",
    "guid": "b8a66384-fc66-4574-a8a9-ad18e610d623",
    "description": "Import point coordinates from generic text files.",
    "inputs": [
      {
        "name": "File",
        "nickname": "F",
        "access": "item",
        "description": "Location of point text file"
      },
      {
        "name": "Separator",
        "nickname": "S",
        "access": "item",
        "description": "Coordinate fragment separator"
      },
      {
        "name": "Comment",
        "nickname": "C",
        "access": "item",
        "description": "Optional comment line start"
      },
      {
        "name": "X Index",
        "nickname": "X",
        "access": "item",
        "description": "Index of point X coordinate"
      },
      {
        "name": "Y Index",
        "nickname": "Y",
        "access": "item",
        "description": "Index of point Y coordinate"
      },
      {
        "name": "Z Index",
        "nickname": "Z",
        "access": "item",
        "description": "Index of point Z coordinate"
      }
    ],
    "outputs": [
      {
        "name": "Points",
        "nickname": "P",
        "access": "list",
        "description": "Imported points"
      }
    ]
  },
  {
    "category": "Params",
    "subcategory": "Input",
    "name": "Import Image",
    "nickname": "IMG",
    "guid": "c2c0c6cf-f362-4047-a159-21a72e7c272a",
    "description": "Import image data from bmp, jpg or png files.",
    "inputs": [
      {
        "name": "File",
        "nickname": "F",
        "access": "item",
        "description": "Location of image file"
      },
      {
        "name": "Rectangle",
        "nickname": "R",
        "access": "item",
        "description": "Optional image destination rectangle"
      },
      {
        "name": "X Samples",
        "nickname": "X",
        "access": "item",
        "description": "Number of samples along image X direction"
      },
      {
        "name": "Y Samples",
        "nickname": "Y",
        "access": "item",
        "description": "Number of samples along image Y direction"
      }
    ],
    "outputs": [
      {
        "name": "Image",
        "nickname": "I",
        "access": "item",
        "description": "A mesh representation of the image"
      }
    ]
  },
  {
    "category": "Params",
    "subcategory": "Input",
    "name": "Object Details",
    "nickname": "ObjDet",
    "guid": "c7b5c66a-6360-4f5f-aa17-a918d0b1c314",
    "description": "Retrieve some details about referenced Rhino objects.",
    "inputs": [
      {
        "name": "Object",
        "nickname": "O",
        "access": "item",
        "description": "Referenced objects"
      }
    ],
    "outputs": [
      {
        "name": "Referenced",
        "nickname": "R",
        "access": "item",
        "description": "Value indicating whether object was referenced."
      },
      {
        "name": "Available",
        "nickname": "A",
        "access": "item",
        "description": "Value indicating whether object was available in the current Rhino document."
      },
      {
        "name": "Name",
        "nickname": "N",
        "access": "item",
        "description": "Object name, if any."
      },
      {
        "name": "Layer",
        "nickname": "L",
        "access": "item",
        "description": "Object layer."
      },
      {
        "name": "Colour",
        "nickname": "C",
        "access": "item",
        "description": "Object display colour resolved within the current document."
      },
      {
        "name": "Guid",
        "nickname": "Id",
        "access": "item",
        "description": "Rhino object id"
      }
    ]
  },
  {
    "category": "Params",
    "subcategory": "Util",
    "name": "Data Input",
    "nickname": "Input",
    "guid": "d8033e3f-8387-4ffc-ab99-929218e8c740",
    "description": "Read a bunch of data from a file.",
    "inputs": [],
    "outputs": []
  },
  {
    "category": "Params",
    "subcategory": "Util",
    "name": "Data Output",
    "nickname": "Output",
    "guid": "dcd23728-8e0e-4241-94b3-7cc3985031dd",
    "description": "Write a bunch of data to a file.",
    "inputs": [
      {
        "name": "Data Input",
        "nickname": "A",
        "access": "tree",
        "description": "Data to include in the file."
      }
    ],
    "outputs": []
  },
  {
    "category": "Params",
    "subcategory": "Input",
    "name": "Import 3DM [OBSOLETE]",
    "nickname": "3DM",
    "guid": "f055c5d7-5d97-4964-90c7-8e9eee9a8a39",
    "description": "This component is OBSOLETE. It has been replaced with a new version.",
    "inputs": [
      {
        "name": "File",
        "nickname": "F",
        "access": "item",
        "description": "Location of Rhino 3dm file"
      },
      {
        "name": "Layer",
        "nickname": "L",
        "access": "item",
        "description": "Layer name filter"
      },
      {
        "name": "Name",
        "nickname": "N",
        "access": "item",
        "description": "Object name filter"
      }
    ],
    "outputs": [
      {
        "name": "Geometry",
        "nickname": "G",
        "access": "list",
        "description": "Imported geometry"
      }
    ]
  },
  {
    "category": "Params",
    "subcategory": "Util",
    "name": "Cluster",
    "nickname": "Cluster",
    "guid": "f31d8d7a-7536-4ac8-9c96-fde6ecda4d0a",
    "description": "Contains a cluster of Grasshopper components",
    "inputs": [],
    "outputs": []
  },
  {
    "category": "Params",
    "subcategory": "Util",
    "name": "Fitness Landscape",
    "nickname": "LScape",
    "guid": "fe9db51e-1ac6-4298-b9dc-6acf3008c8f2",
    "description": "Display a 2.5D fitness landscape",
    "inputs": [
      {
        "name": "Bounds",
        "nickname": "B",
        "access": "item",
        "description": "Landscape bounds"
      },
      {
        "name": "Values",
        "nickname": "V",
        "access": "list",
        "description": "Landscape values"
      },
      {
        "name": "Count",
        "nickname": "N",
        "access": "item",
        "description": "Number of samples along X direction"
      }
    ],
    "outputs": [
      {
        "name": "Landscape",
        "nickname": "L",
        "access": "item",
        "description": "Landscaper mesh"
      }
    ]
  }
];
