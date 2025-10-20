export const MATHS_COMPONENTS = [
  {
    "category": "Maths",
    "subcategory": "Script",
    "name": "F2",
    "nickname": "F(x,y)",
    "guid": "00ec9ecd-4e1d-45ba-a8fc-dff716dbd9e4",
    "description": "A function of two variables; {x,y}",
    "inputs": [
      {
        "name": "Function",
        "nickname": "F",
        "access": "item",
        "description": "Expression to solve"
      },
      {
        "name": "x",
        "nickname": "x",
        "access": "item",
        "description": "Variable #1"
      },
      {
        "name": "y",
        "nickname": "y",
        "access": "item",
        "description": "Variable #2"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "r",
        "access": "item",
        "description": "Expression result"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Operators",
    "name": "Gate And",
    "nickname": "And",
    "guid": "040f195d-0b4e-4fe0-901f-fedb2fd3db15",
    "description": "Perform boolean conjunction (AND gate).",
    "inputs": [
      {
        "name": "A",
        "nickname": "A",
        "access": "item",
        "description": "First boolean for AND operation"
      },
      {
        "name": "B",
        "nickname": "B",
        "access": "item",
        "description": "Second boolean for AND operation"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "R",
        "access": "item",
        "description": "Resulting value"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Script",
    "name": "VB Script",
    "nickname": "VB",
    "guid": "079bd9bd-54a0-41d4-98af-db999015f63d",
    "description": "A VB.NET scriptable component",
    "inputs": [
      {
        "name": "x",
        "nickname": "x",
        "access": "item",
        "description": "Script Variable x"
      },
      {
        "name": "y",
        "nickname": "y",
        "access": "item",
        "description": "Script Variable y"
      }
    ],
    "outputs": [
      {
        "name": "out",
        "nickname": "out",
        "access": "list",
        "description": "Print, Reflect and Error streams"
      },
      {
        "name": "A",
        "nickname": "A",
        "access": "item",
        "description": "Output parameter A"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Script",
    "name": "F4",
    "nickname": "F(a,b,c,d)",
    "guid": "07efd5e1-d7f4-4205-ab99-83e68175564e",
    "description": "A function of four variables; {a,b,c,d}.",
    "inputs": [
      {
        "name": "Function",
        "nickname": "F",
        "access": "item",
        "description": "Expression to solve"
      },
      {
        "name": "a",
        "nickname": "a",
        "access": "item",
        "description": "Variable #1"
      },
      {
        "name": "b",
        "nickname": "b",
        "access": "item",
        "description": "Variable #2"
      },
      {
        "name": "c",
        "nickname": "c",
        "access": "item",
        "description": "Variable #3"
      },
      {
        "name": "d",
        "nickname": "d",
        "access": "item",
        "description": "Variable #4"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "r",
        "access": "item",
        "description": "Expression result"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Domain",
    "name": "Find Domain",
    "nickname": "FDom",
    "guid": "0b5c7fad-0473-41aa-bf52-d7a861dcaa29",
    "description": "Find the first domain that contains a specific value",
    "inputs": [
      {
        "name": "Domains",
        "nickname": "D",
        "access": "list",
        "description": "Collection of domains to search"
      },
      {
        "name": "Number",
        "nickname": "N",
        "access": "item",
        "description": "Number to test"
      },
      {
        "name": "Strict",
        "nickname": "S",
        "access": "item",
        "description": "Strict comparison, if true then the value must be on the interior of a domain"
      }
    ],
    "outputs": [
      {
        "name": "Index",
        "nickname": "I",
        "access": "item",
        "description": "Index of first domain that includes the specified value"
      },
      {
        "name": "Neighbour",
        "nickname": "N",
        "access": "item",
        "description": "Index of domain that is closest to the specified value"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Script",
    "name": "F1",
    "nickname": "F(x)",
    "guid": "0b7d1129-7b88-4322-aad3-56fd1036a8f6",
    "description": "A function of a single variable; {x}.",
    "inputs": [
      {
        "name": "Function",
        "nickname": "F",
        "access": "item",
        "description": "Expression to solve"
      },
      {
        "name": "x",
        "nickname": "x",
        "access": "item",
        "description": "Variable #1"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "r",
        "access": "item",
        "description": "Expression result"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Time",
    "name": "Construct Date",
    "nickname": "Date",
    "guid": "0c2f0932-5ddc-4ece-bd84-a3a059d3df7a",
    "description": "Construct a date and time instance.",
    "inputs": [
      {
        "name": "Year",
        "nickname": "Y",
        "access": "item",
        "description": "Year number (must be between 1 and 9999)"
      },
      {
        "name": "Month",
        "nickname": "M",
        "access": "item",
        "description": "Month number (must be between 1 and 12)"
      },
      {
        "name": "Day",
        "nickname": "D",
        "access": "item",
        "description": "Day of month (must be between 1 and 31)"
      },
      {
        "name": "Hour",
        "nickname": "h",
        "access": "item",
        "description": "Hour of day (must be between 0 and 23)"
      },
      {
        "name": "Minute",
        "nickname": "m",
        "access": "item",
        "description": "Minute of the hour (must be between 0 and 59)"
      },
      {
        "name": "Second",
        "nickname": "s",
        "access": "item",
        "description": "Second of the minute (must be between 0 and 59)"
      }
    ],
    "outputs": [
      {
        "name": "Date",
        "nickname": "D",
        "access": "item",
        "description": "Date and Time data"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Util",
    "name": "Maximum",
    "nickname": "Max",
    "guid": "0d1e2027-f153-460d-84c0-f9af431b08cb",
    "description": "Return the greater of two items.",
    "inputs": [
      {
        "name": "A",
        "nickname": "A",
        "access": "item",
        "description": "First item for comparison"
      },
      {
        "name": "B",
        "nickname": "B",
        "access": "item",
        "description": "Second item for comparison"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "R",
        "access": "item",
        "description": "The greater of A and B"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Util",
    "name": "Pi",
    "nickname": "Pi",
    "guid": "0d2ccfb3-9d41-4759-9452-da6a522c3eaa",
    "description": "Returns a factor of Pi.",
    "inputs": [
      {
        "name": "Factor",
        "nickname": "N",
        "access": "item",
        "description": "Factor to be multiplied by Pi"
      }
    ],
    "outputs": [
      {
        "name": "Output",
        "nickname": "y",
        "access": "item",
        "description": "Output value"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Trig",
    "name": "Degrees",
    "nickname": "Deg",
    "guid": "0d77c51e-584f-44e8-aed2-c2ddf4803888",
    "description": "Convert an angle specified in radians to degrees",
    "inputs": [
      {
        "name": "Radians",
        "nickname": "R",
        "access": "item",
        "description": "Angle in radians"
      }
    ],
    "outputs": [
      {
        "name": "Degrees",
        "nickname": "D",
        "access": "item",
        "description": "Angle in degrees"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Matrix",
    "name": "Transpose Matrix",
    "nickname": "Transpose",
    "guid": "0e90b1f3-b870-4e09-8711-4bf819675d90",
    "description": "Transpose a matrix (swap rows and columns)",
    "inputs": [
      {
        "name": "Matrix",
        "nickname": "M",
        "access": "item",
        "description": "A newly created matrix"
      }
    ],
    "outputs": [
      {
        "name": "Matrix",
        "nickname": "M",
        "access": "item",
        "description": "Transposed matrix"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Trig",
    "name": "Tangent",
    "nickname": "Tan",
    "guid": "0f31784f-7177-4104-8500-1f4f4a306df4",
    "description": "Compute the tangent of a value",
    "inputs": [
      {
        "name": "Value",
        "nickname": "x",
        "access": "item",
        "description": "Input value"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "y",
        "access": "item",
        "description": "Output value"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Script",
    "name": "F(a,b,c,d) [OBSOLETE]",
    "nickname": "F(a,b,c,d)",
    "guid": "0f3a13d4-5bb7-499e-9b57-56bb6dce93fd",
    "description": "A function of four variables a, b, c and d",
    "inputs": [
      {
        "name": "Function",
        "nickname": "F",
        "access": "item",
        "description": "The function script"
      },
      {
        "name": "Variable a",
        "nickname": "a",
        "access": "item",
        "description": "The first variable"
      },
      {
        "name": "Variable b",
        "nickname": "b",
        "access": "item",
        "description": "The second variable"
      },
      {
        "name": "Variable c",
        "nickname": "c",
        "access": "item",
        "description": "The third variable"
      },
      {
        "name": "Variable d",
        "nickname": "d",
        "access": "item",
        "description": "The fourth variable"
      }
    ],
    "outputs": [
      {
        "name": "Result R",
        "nickname": "R",
        "access": "item",
        "description": "Equation solution"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Script",
    "name": "VB Script",
    "nickname": "VB",
    "guid": "1e9e08fc-c31e-49eb-a36c-90de5e62e5f5",
    "description": "A VB.NET scriptable component",
    "inputs": [
      {
        "name": "Variable x",
        "nickname": "x",
        "access": "item",
        "description": "Script Variable x"
      },
      {
        "name": "Variable y",
        "nickname": "y",
        "access": "item",
        "description": "Script Variable y"
      }
    ],
    "outputs": [
      {
        "name": "Output",
        "nickname": "out",
        "access": "item",
        "description": "Print, Reflect and Error streams"
      },
      {
        "name": "Result A",
        "nickname": "A",
        "access": "item",
        "description": "Output parameter A"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Util",
    "name": "Complex Components",
    "nickname": "Complex",
    "guid": "1f384257-b26b-4160-a6d3-1dcd89b64acd",
    "description": "Extract the Real and Imaginary components from a complex number",
    "inputs": [
      {
        "name": "Complex",
        "nickname": "C",
        "access": "item",
        "description": "Complex number to disembowel"
      }
    ],
    "outputs": [
      {
        "name": "Real",
        "nickname": "R",
        "access": "item",
        "description": "Real component of complex number"
      },
      {
        "name": "Imaginary",
        "nickname": "i",
        "access": "item",
        "description": "Imaginary component of complex number"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Trig",
    "name": "CoTangent",
    "nickname": "Cot",
    "guid": "1f602c33-f38e-4f47-898b-359f0a4de3c2",
    "description": "Compute the co-tangent (reciprocal of the Tangent) of an angle.",
    "inputs": [
      {
        "name": "Value",
        "nickname": "x",
        "access": "item",
        "description": "Input value"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "y",
        "access": "item",
        "description": "Output value"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Trig",
    "name": "Circumcentre",
    "nickname": "CCentre",
    "guid": "21d0767c-5340-4087-aa09-398d0e706908",
    "description": "Generate the triangle circumcentre from perpendicular bisectors.",
    "inputs": [
      {
        "name": "Point A",
        "nickname": "A",
        "access": "item",
        "description": "First triangle corner"
      },
      {
        "name": "Point B",
        "nickname": "B",
        "access": "item",
        "description": "Second triangle corner"
      },
      {
        "name": "Point C",
        "nickname": "C",
        "access": "item",
        "description": "Third triangle corner"
      }
    ],
    "outputs": [
      {
        "name": "Circumcentre",
        "nickname": "C",
        "access": "item",
        "description": "Circumcentre point for triangle"
      },
      {
        "name": "Bisector AB",
        "nickname": "AB",
        "access": "item",
        "description": "Perpendicular bisector line emanating from edge AB"
      },
      {
        "name": "Bisector BC",
        "nickname": "BC",
        "access": "item",
        "description": "Perpendicular bisector line emanating from edge AB"
      },
      {
        "name": "Bisector CA",
        "nickname": "CA",
        "access": "item",
        "description": "Perpendicular bisector line emanating from edge AB"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Polynomials",
    "name": "Square",
    "nickname": "Sqr",
    "guid": "2280dde4-9fa2-4b4a-ae2f-37d554861367",
    "description": "Compute the square of a value",
    "inputs": [
      {
        "name": "Value",
        "nickname": "x",
        "access": "item",
        "description": "Input value"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "y",
        "access": "item",
        "description": "Output value"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Polynomials",
    "name": "Natural logarithm",
    "nickname": "Ln",
    "guid": "23afc7aa-2d2f-4ae7-b876-bf366246b826",
    "description": "Compute the natural logarithm of a value.",
    "inputs": [
      {
        "name": "Value",
        "nickname": "x",
        "access": "item",
        "description": "Input value"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "y",
        "access": "item",
        "description": "Output value"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Polynomials",
    "name": "Logarithm",
    "nickname": "Log",
    "guid": "27d6f724-a701-4585-992f-3897488abf08",
    "description": "Compute the Base-10 logarithm of a value.",
    "inputs": [
      {
        "name": "Value",
        "nickname": "x",
        "access": "item",
        "description": "Input value"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "y",
        "access": "item",
        "description": "Output value"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Operators",
    "name": "Absolute",
    "nickname": "Abs",
    "guid": "28124995-cf99-4298-b6f4-c75a8e379f18",
    "description": "Compute the absolute of a value.",
    "inputs": [
      {
        "name": "Value",
        "nickname": "x",
        "access": "item",
        "description": "Input value"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "y",
        "access": "item",
        "description": "Output value"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Boolean",
    "name": "Gate And",
    "nickname": "And",
    "guid": "28f35e12-cd50-4bce-b036-695c2a3d04da",
    "description": "Perform boolean conjunction (AND gate).",
    "inputs": [
      {
        "name": "A",
        "nickname": "A",
        "access": "item",
        "description": "Left hand boolean"
      },
      {
        "name": "B",
        "nickname": "B",
        "access": "item",
        "description": "Right hand boolean"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "R",
        "access": "item",
        "description": "Resulting value"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Operators",
    "name": "Subtraction",
    "nickname": "A-B",
    "guid": "2c56ab33-c7cc-4129-886c-d5856b714010",
    "description": "Mathematical subtraction",
    "inputs": [
      {
        "name": "A",
        "nickname": "A",
        "access": "item",
        "description": "Item to subtract from (minuend)"
      },
      {
        "name": "B",
        "nickname": "B",
        "access": "item",
        "description": "Item to subtract (subtrahend)"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "R",
        "access": "item",
        "description": "The result of the Subtraction"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Polynomials",
    "name": "Power of 10",
    "nickname": "10º",
    "guid": "2ebb82ef-1f90-4ac9-9a71-1fe0f4ef7044",
    "description": "Raise 10 to the power of N.",
    "inputs": [
      {
        "name": "Value",
        "nickname": "x",
        "access": "item",
        "description": "Input value"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "y",
        "access": "item",
        "description": "Output value"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Script",
    "name": "F3",
    "nickname": "F(x,y,z)",
    "guid": "2f77b45b-034d-4053-8872-f38d87cbc676",
    "description": "A function of three variables; {x,y,z}.",
    "inputs": [
      {
        "name": "Function",
        "nickname": "F",
        "access": "item",
        "description": "Expression to solve"
      },
      {
        "name": "x",
        "nickname": "x",
        "access": "item",
        "description": "Variable #1"
      },
      {
        "name": "y",
        "nickname": "y",
        "access": "item",
        "description": "Variable #2"
      },
      {
        "name": "z",
        "nickname": "z",
        "access": "item",
        "description": "Variable #3"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "r",
        "access": "item",
        "description": "Expression result"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Domain",
    "name": "Remap Numbers",
    "nickname": "ReMap",
    "guid": "2fcc2743-8339-4cdf-a046-a1f17439191d",
    "description": "Remap numbers into a new numeric domain",
    "inputs": [
      {
        "name": "Value",
        "nickname": "V",
        "access": "item",
        "description": "Value to remap"
      },
      {
        "name": "Source",
        "nickname": "S",
        "access": "item",
        "description": "Source domain"
      },
      {
        "name": "Target",
        "nickname": "T",
        "access": "item",
        "description": "Target domain"
      }
    ],
    "outputs": [
      {
        "name": "Mapped",
        "nickname": "R",
        "access": "item",
        "description": "Remapped number"
      },
      {
        "name": "Clipped",
        "nickname": "C",
        "access": "item",
        "description": "Remapped and clipped number"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Operators",
    "name": "Larger Than",
    "nickname": "Larger",
    "guid": "30d58600-1aab-42db-80a3-f1ea6c4269a0",
    "description": "Larger than (or equal to)",
    "inputs": [
      {
        "name": "First Number",
        "nickname": "A",
        "access": "item",
        "description": "Number to test"
      },
      {
        "name": "Second Number",
        "nickname": "B",
        "access": "item",
        "description": "Number to test against"
      }
    ],
    "outputs": [
      {
        "name": "Larger than",
        "nickname": ">",
        "access": "item",
        "description": "True if A > B"
      },
      {
        "name": "… or Equal to",
        "nickname": ">=",
        "access": "item",
        "description": "True if A >= B"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Time",
    "name": "Combine Date & Time",
    "nickname": "CDate",
    "guid": "31534405-6573-4be6-8bf8-262e55847a3a",
    "description": "Combine a pure date and a pure time into a single date",
    "inputs": [
      {
        "name": "Date",
        "nickname": "D",
        "access": "item",
        "description": "Date portion"
      },
      {
        "name": "Time",
        "nickname": "T",
        "access": "item",
        "description": "Time portion"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "R",
        "access": "item",
        "description": "Resulting combination of date and time."
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Script",
    "name": "F5",
    "nickname": "F(a,b,c,d,x)",
    "guid": "322f0e6e-d434-4d07-9f8d-f214bb248cb1",
    "description": "A function of five variables; {a,b,c,d,x}.",
    "inputs": [
      {
        "name": "Function",
        "nickname": "F",
        "access": "item",
        "description": "Expression to solve"
      },
      {
        "name": "a",
        "nickname": "a",
        "access": "item",
        "description": "Variable #1"
      },
      {
        "name": "b",
        "nickname": "b",
        "access": "item",
        "description": "Variable #2"
      },
      {
        "name": "c",
        "nickname": "c",
        "access": "item",
        "description": "Variable #3"
      },
      {
        "name": "d",
        "nickname": "d",
        "access": "item",
        "description": "Variable #4"
      },
      {
        "name": "x",
        "nickname": "x",
        "access": "item",
        "description": "Variable #5"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "r",
        "access": "item",
        "description": "Expression result"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Util",
    "name": "Weighted Average",
    "nickname": "Wav",
    "guid": "338666eb-14c5-4d9b-82e2-2b5be60655df",
    "description": "Solve the arithmetic weighted average for a set of items",
    "inputs": [
      {
        "name": "Input",
        "nickname": "I",
        "access": "list",
        "description": "Input values for averaging"
      },
      {
        "name": "Weights",
        "nickname": "W",
        "access": "list",
        "description": "Collection of weights for each value"
      }
    ],
    "outputs": [
      {
        "name": "Arithmetic mean",
        "nickname": "AM",
        "access": "item",
        "description": "Arithmetic mean (average) of all input values"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Trig",
    "name": "Orthocentre",
    "nickname": "OCentre",
    "guid": "36dd5551-b6bd-4246-bd2f-1fd91eb2f02d",
    "description": "Generate the triangle orthocentre from altitudes.",
    "inputs": [
      {
        "name": "Point A",
        "nickname": "A",
        "access": "item",
        "description": "First triangle corner"
      },
      {
        "name": "Point B",
        "nickname": "B",
        "access": "item",
        "description": "Second triangle corner"
      },
      {
        "name": "Point C",
        "nickname": "C",
        "access": "item",
        "description": "Third triangle corner"
      }
    ],
    "outputs": [
      {
        "name": "Orthocentre",
        "nickname": "C",
        "access": "item",
        "description": "Orthocentre point for triangle"
      },
      {
        "name": "Altitude AB",
        "nickname": "AB",
        "access": "item",
        "description": "Altitude line connecting edge AB with corner C"
      },
      {
        "name": "Altitude BC",
        "nickname": "BC",
        "access": "item",
        "description": "Altitude line connecting edge BC with corner A"
      },
      {
        "name": "Altitude CA",
        "nickname": "CA",
        "access": "item",
        "description": "Altitude line connecting edge CA with corner B"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Util",
    "name": "Extremes",
    "nickname": "Extrz",
    "guid": "37084b3f-2b66-4f3a-9737-80d0b0b7f0cb",
    "description": "Find the extremes in a list of values",
    "inputs": [
      {
        "name": "A",
        "nickname": "A",
        "access": "item",
        "description": "Value for comparison"
      },
      {
        "name": "B",
        "nickname": "B",
        "access": "item",
        "description": "Value for comparison"
      }
    ],
    "outputs": [
      {
        "name": "Mininum",
        "nickname": "V-",
        "access": "item",
        "description": "Lowest of all values"
      },
      {
        "name": "Maximum",
        "nickname": "V+",
        "access": "item",
        "description": "Highest of all values"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Time",
    "name": "Date Range",
    "nickname": "RDate",
    "guid": "38a4e722-ad5a-4229-a170-e27ae1345538",
    "description": "Create a range of successive dates or times",
    "inputs": [
      {
        "name": "Time A",
        "nickname": "A",
        "access": "item",
        "description": "First time"
      },
      {
        "name": "Time B",
        "nickname": "B",
        "access": "item",
        "description": "Second time"
      },
      {
        "name": "Count",
        "nickname": "N",
        "access": "item",
        "description": "Number of times to create between A and B"
      }
    ],
    "outputs": [
      {
        "name": "Range",
        "nickname": "R",
        "access": "list",
        "description": "Range of varying times between A and B."
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Matrix",
    "name": "Deconstruct Matrix",
    "nickname": "DeMatrix",
    "guid": "3aa2a080-e322-4be3-8c6e-baf6c8000cf1",
    "description": "Deconstruct a matrix into its component parts",
    "inputs": [
      {
        "name": "Matrix",
        "nickname": "M",
        "access": "item",
        "description": "Matrix to deconstruct"
      }
    ],
    "outputs": [
      {
        "name": "Rows",
        "nickname": "R",
        "access": "item",
        "description": "Number of rows in the matrix"
      },
      {
        "name": "Columns",
        "nickname": "C",
        "access": "item",
        "description": "Number of columns in the matrix"
      },
      {
        "name": "Values",
        "nickname": "V",
        "access": "list",
        "description": "Matrix values"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Operators",
    "name": "Similarity",
    "nickname": "Similar",
    "guid": "40177d8a-a35c-4622-bca7-d150031fe427",
    "description": "Test for similarity of two numbers",
    "inputs": [
      {
        "name": "First Number",
        "nickname": "A",
        "access": "item",
        "description": "Number to compare"
      },
      {
        "name": "Second Number",
        "nickname": "B",
        "access": "item",
        "description": "Number to compare to"
      },
      {
        "name": "Threshold",
        "nickname": "T%",
        "access": "item",
        "description": "Percentage (0% ~ 100%) of A and B below which similarity is assumed"
      }
    ],
    "outputs": [
      {
        "name": "Similarity",
        "nickname": "=",
        "access": "item",
        "description": "True if A ≈ B"
      },
      {
        "name": "Absolute difference",
        "nickname": "dt",
        "access": "item",
        "description": "The absolute difference between A and B"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Time",
    "name": "Interpolate Date",
    "nickname": "IntDate",
    "guid": "4083802b-3dd9-4b13-9756-bf5441213e70",
    "description": "Interpolate between two dates or times.",
    "inputs": [
      {
        "name": "Date A",
        "nickname": "A",
        "access": "item",
        "description": "First date"
      },
      {
        "name": "Date B",
        "nickname": "B",
        "access": "item",
        "description": "Second date"
      },
      {
        "name": "Interpolation",
        "nickname": "t",
        "access": "item",
        "description": "Interpolation factor"
      }
    ],
    "outputs": [
      {
        "name": "Date",
        "nickname": "D",
        "access": "item",
        "description": "Interpolated Date & Time"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Script",
    "name": "GhPython Script",
    "nickname": "Python",
    "guid": "410755b1-224a-4c1e-a407-bf32fb45ea7e",
    "description": "GhPython provides a Python script component",
    "inputs": [
      {
        "name": "x",
        "nickname": "x",
        "access": "item",
        "description": "Script variable Python"
      },
      {
        "name": "y",
        "nickname": "y",
        "access": "item",
        "description": "Script variable Python"
      }
    ],
    "outputs": [
      {
        "name": "out",
        "nickname": "out",
        "access": "item",
        "description": "The execution information, as output and error streams"
      },
      {
        "name": "a",
        "nickname": "a",
        "access": "item",
        "description": "Script variable Python"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Operators",
    "name": "Modulus",
    "nickname": "Mod",
    "guid": "431bc610-8ae1-4090-b217-1a9d9c519fe2",
    "description": "Divides two numbers and returns only the remainder.",
    "inputs": [
      {
        "name": "A",
        "nickname": "A",
        "access": "item",
        "description": "First number for modulo (dividend)"
      },
      {
        "name": "B",
        "nickname": "B",
        "access": "item",
        "description": "Second number for modulo (divisor)"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "R",
        "access": "item",
        "description": "The remainder of A/B"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Script",
    "name": "F6",
    "nickname": "F(a,b,c,d,x,y)",
    "guid": "4783b96f-6197-4058-a688-b4ba04c00962",
    "description": "A function of six variables; {a,b,c,d,x,y}.",
    "inputs": [
      {
        "name": "Function",
        "nickname": "F",
        "access": "item",
        "description": "Expression to solve"
      },
      {
        "name": "a",
        "nickname": "a",
        "access": "item",
        "description": "Variable #1"
      },
      {
        "name": "b",
        "nickname": "b",
        "access": "item",
        "description": "Variable #2"
      },
      {
        "name": "c",
        "nickname": "c",
        "access": "item",
        "description": "Variable #3"
      },
      {
        "name": "d",
        "nickname": "d",
        "access": "item",
        "description": "Variable #4"
      },
      {
        "name": "x",
        "nickname": "x",
        "access": "item",
        "description": "Variable #5"
      },
      {
        "name": "y",
        "nickname": "y",
        "access": "item",
        "description": "Variable #6"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "r",
        "access": "item",
        "description": "Expression result"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Domain",
    "name": "Deconstruct Domain²",
    "nickname": "DeDom2Num",
    "guid": "47c30f9d-b685-4d4d-9b20-5b60e48d5af8",
    "description": "Deconstruct a two-dimensional domain into four numbers",
    "inputs": [
      {
        "name": "Domain",
        "nickname": "I",
        "access": "item",
        "description": "Base domain"
      }
    ],
    "outputs": [
      {
        "name": "U min",
        "nickname": "U0",
        "access": "item",
        "description": "Lower limit of domain in {u} direction"
      },
      {
        "name": "U max",
        "nickname": "U1",
        "access": "item",
        "description": "Upper limit of domain in {u} direction"
      },
      {
        "name": "V min",
        "nickname": "V0",
        "access": "item",
        "description": "Lower limit of domain in {v} direction"
      },
      {
        "name": "V max",
        "nickname": "V1",
        "access": "item",
        "description": "Upper limit of domain in {v} direction"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Trig",
    "name": "ArcCosine",
    "nickname": "ACos",
    "guid": "49584390-d541-41f7-b5f6-1f9515ac0f73",
    "description": "Compute the angle whose cosine is the specified value.",
    "inputs": [
      {
        "name": "Value",
        "nickname": "x",
        "access": "item",
        "description": "Input value"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "y",
        "access": "item",
        "description": "Output value"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Matrix",
    "name": "Swap Columns",
    "nickname": "SwapC",
    "guid": "4cebcaf7-9a6a-435b-8f8f-95a62bacb0f2",
    "description": "Swap two columns in a matrix",
    "inputs": [
      {
        "name": "Matrix",
        "nickname": "M",
        "access": "item",
        "description": "Matrix for column swap"
      },
      {
        "name": "Column A",
        "nickname": "A",
        "access": "item",
        "description": "First column index"
      },
      {
        "name": "Column B",
        "nickname": "B",
        "access": "item",
        "description": "Second column index"
      }
    ],
    "outputs": [
      {
        "name": "Matrix",
        "nickname": "M",
        "access": "item",
        "description": "Matrix with swapped rows"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Operators",
    "name": "Gate Nor",
    "nickname": "Nor",
    "guid": "548177c2-d1db-4172-b667-bec979e2d38b",
    "description": "Perform boolean joint denial (NOR gate).",
    "inputs": [
      {
        "name": "A",
        "nickname": "A",
        "access": "item",
        "description": "Left hand boolean"
      },
      {
        "name": "B",
        "nickname": "B",
        "access": "item",
        "description": "Right hand boolean"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "R",
        "access": "item",
        "description": "Resulting value"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Matrix",
    "name": "Construct Matrix",
    "nickname": "Matrix",
    "guid": "54ac80cf-74f3-43f7-834c-0e3fe94632c6",
    "description": "Construct a matrix from initial values",
    "inputs": [
      {
        "name": "Rows",
        "nickname": "R",
        "access": "item",
        "description": "Number of rows in the matrix"
      },
      {
        "name": "Columns",
        "nickname": "C",
        "access": "item",
        "description": "Number of columns in the matrix"
      },
      {
        "name": "Values",
        "nickname": "V",
        "access": "list",
        "description": "Optional matrix values, if omitted, an identity matrix will be created"
      }
    ],
    "outputs": [
      {
        "name": "Matrix",
        "nickname": "M",
        "access": "item",
        "description": "A newly created matrix"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Operators",
    "name": "Integer Division",
    "nickname": "A\\B",
    "guid": "54db2568-3441-4ae2-bcef-92c4cc608e11",
    "description": "Mathematical integer division",
    "inputs": [
      {
        "name": "A",
        "nickname": "A",
        "access": "item",
        "description": "Item to divide (dividend)"
      },
      {
        "name": "B",
        "nickname": "B",
        "access": "item",
        "description": "Item to divide with (divisor)"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "R",
        "access": "item",
        "description": "Result of integer division"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Boolean",
    "name": "Gate Or Ternary",
    "nickname": "Or",
    "guid": "55104772-8096-4ffc-a78a-30e36191ace2",
    "description": "Perform ternary boolean disjunction (Or gate).",
    "inputs": [
      {
        "name": "A",
        "nickname": "A",
        "access": "item",
        "description": "First boolean"
      },
      {
        "name": "B",
        "nickname": "B",
        "access": "item",
        "description": "Second boolean"
      },
      {
        "name": "C",
        "nickname": "C",
        "access": "item",
        "description": "Third boolean"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "R",
        "access": "item",
        "description": "Resulting value"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Util",
    "name": "Minimum",
    "nickname": "Min",
    "guid": "57308b30-772d-4919-ac67-e86c18f3a996",
    "description": "Return the lesser of two items.",
    "inputs": [
      {
        "name": "A",
        "nickname": "A",
        "access": "item",
        "description": "First item for comparison"
      },
      {
        "name": "B",
        "nickname": "B",
        "access": "item",
        "description": "Second item for comparison"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "R",
        "access": "item",
        "description": "The lesser of A and B"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Script",
    "name": "Eval [OBSOLETE]",
    "nickname": "Eval",
    "guid": "579c9f8c-6fb6-419b-8086-523a2dc99e8a",
    "description": "Evaluate an expression",
    "inputs": [
      {
        "name": "Expression",
        "nickname": "E",
        "access": "item",
        "description": "The expression to evaluate"
      }
    ],
    "outputs": [
      {
        "name": "Value",
        "nickname": "V",
        "access": "item",
        "description": "Expression value"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Util",
    "name": "Blur Numbers",
    "nickname": "NBlur",
    "guid": "57e1d392-e3fb-4de9-be98-982854a92351",
    "description": "Blur a list of numbers by averaging neighbours",
    "inputs": [
      {
        "name": "Numbers",
        "nickname": "N",
        "access": "list",
        "description": "Numbers to blur"
      },
      {
        "name": "Strength",
        "nickname": "S",
        "access": "item",
        "description": "Blurring strength (0=none, 1=full)"
      },
      {
        "name": "Iterations",
        "nickname": "I",
        "access": "item",
        "description": "Number of successive blurring iterations"
      },
      {
        "name": "Lock",
        "nickname": "L",
        "access": "item",
        "description": "Lock first and last value"
      },
      {
        "name": "Wrap",
        "nickname": "W",
        "access": "item",
        "description": "Treat the list as a cyclical collection"
      }
    ],
    "outputs": [
      {
        "name": "Numbers",
        "nickname": "N",
        "access": "list",
        "description": "Blurred numbers"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Operators",
    "name": "Series Addition",
    "nickname": "SA",
    "guid": "586706a8-109b-43ec-b581-743e920c951a",
    "description": "Perform serial addition until a goal has been reached",
    "inputs": [
      {
        "name": "Numbers",
        "nickname": "N",
        "access": "list",
        "description": "Number pool from which to take summands"
      },
      {
        "name": "Goal",
        "nickname": "G",
        "access": "item",
        "description": "Goal value of addition series"
      },
      {
        "name": "Start",
        "nickname": "S",
        "access": "item",
        "description": "Starting value of addition series"
      }
    ],
    "outputs": [
      {
        "name": "Series",
        "nickname": "S",
        "access": "list",
        "description": "Addition series"
      },
      {
        "name": "Remainder",
        "nickname": "R",
        "access": "item",
        "description": "Difference between series summation and goal"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Time",
    "name": "Construct Time",
    "nickname": "Time",
    "guid": "595aded2-8916-402d-87a3-a825244bbe3d",
    "description": "Construct a time instance",
    "inputs": [
      {
        "name": "Hour",
        "nickname": "H",
        "access": "item",
        "description": "Number of hours"
      },
      {
        "name": "Minute",
        "nickname": "M",
        "access": "item",
        "description": "Number of minutes"
      },
      {
        "name": "Second",
        "nickname": "S",
        "access": "item",
        "description": "Number of seconds"
      }
    ],
    "outputs": [
      {
        "name": "Time",
        "nickname": "T",
        "access": "item",
        "description": "Time construct"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Polynomials",
    "name": "Cube Root",
    "nickname": "Cbrt",
    "guid": "5b0be57a-31f5-4446-a11a-ae0d348bca90",
    "description": "Compute the cube root of a value",
    "inputs": [
      {
        "name": "Value",
        "nickname": "x",
        "access": "item",
        "description": "Input value"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "y",
        "access": "item",
        "description": "Output value"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Util",
    "name": "Smooth Numbers",
    "nickname": "Smooth",
    "guid": "5b424e1c-d061-43cd-8c20-db84564b0502",
    "description": "Smooth out changing numbers over time",
    "inputs": [
      {
        "name": "Numbers",
        "nickname": "N",
        "access": "tree",
        "description": "Changing numbers"
      }
    ],
    "outputs": [
      {
        "name": "Numbers",
        "nickname": "N",
        "access": "tree",
        "description": "Smoothened numbers"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Operators",
    "name": "Mass Addition",
    "nickname": "MA",
    "guid": "5b850221-b527-4bd6-8c62-e94168cd6efa",
    "description": "Perform mass addition of a list of items",
    "inputs": [
      {
        "name": "Input",
        "nickname": "I",
        "access": "list",
        "description": "Input values for mass addition."
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "R",
        "access": "item",
        "description": "Result of mass addition"
      },
      {
        "name": "Partial Results",
        "nickname": "Pr",
        "access": "list",
        "description": "List of partial results"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Operators",
    "name": "Gate Nand",
    "nickname": "Nand",
    "guid": "5ca5de6b-bc71-46c4-a8f7-7f30d7040acb",
    "description": "Perform boolean alternative denial (NAND gate).",
    "inputs": [
      {
        "name": "A",
        "nickname": "A",
        "access": "item",
        "description": "Left hand boolean"
      },
      {
        "name": "B",
        "nickname": "B",
        "access": "item",
        "description": "Right hand boolean"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "R",
        "access": "item",
        "description": "Resulting value"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Operators",
    "name": "Gate Or",
    "nickname": "Or",
    "guid": "5cad70f9-5a53-4c5c-a782-54a479b4abe3",
    "description": "Perform boolean disjunction (OR gate).",
    "inputs": [
      {
        "name": "A",
        "nickname": "A",
        "access": "item",
        "description": "First boolean for OR operation"
      },
      {
        "name": "B",
        "nickname": "B",
        "access": "item",
        "description": "Second boolean for OR operation"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "R",
        "access": "item",
        "description": "Resulting value"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Operators",
    "name": "Equality",
    "nickname": "Equals",
    "guid": "5db0fb89-4f22-4f09-a777-fa5e55aed7ec",
    "description": "Test for (in)equality of two numbers",
    "inputs": [
      {
        "name": "First Number",
        "nickname": "A",
        "access": "item",
        "description": "Number to compare"
      },
      {
        "name": "Second Number",
        "nickname": "B",
        "access": "item",
        "description": "Number to compare to"
      }
    ],
    "outputs": [
      {
        "name": "Equality",
        "nickname": "=",
        "access": "item",
        "description": "True if A = B"
      },
      {
        "name": "Inequality",
        "nickname": "≠",
        "access": "item",
        "description": "True if A ≠ B"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Trig",
    "name": "Secant",
    "nickname": "Sec",
    "guid": "60103def-1bb7-4700-b294-3a89100525c4",
    "description": "Compute the secant (reciprocal of the Cosine) of an angle.",
    "inputs": [
      {
        "name": "Value",
        "nickname": "x",
        "access": "item",
        "description": "Input value"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "y",
        "access": "item",
        "description": "Output value"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Util",
    "name": "Create Complex",
    "nickname": "Complex",
    "guid": "63d12974-2915-4ccf-ac26-5d566c3bac92",
    "description": "Create a complex number from a Real and an Imaginary component",
    "inputs": [
      {
        "name": "Real",
        "nickname": "R",
        "access": "item",
        "description": "Real component of complex number"
      },
      {
        "name": "Imaginary",
        "nickname": "i",
        "access": "item",
        "description": "Imaginary component of complex number"
      }
    ],
    "outputs": [
      {
        "name": "Complex",
        "nickname": "C",
        "access": "item",
        "description": "Complex number"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Script",
    "name": "Evaluate Expression [OBSOLETE]",
    "nickname": "Eval",
    "guid": "655c5f2f-1e40-42b8-a93a-f05032794449",
    "description": "Evaluate an expression",
    "inputs": [
      {
        "name": "Expression",
        "nickname": "E",
        "access": "item",
        "description": "The expression to evaluate"
      },
      {
        "name": "Variable a",
        "nickname": "a",
        "access": "item",
        "description": "The first variable"
      },
      {
        "name": "Variable b",
        "nickname": "b",
        "access": "item",
        "description": "The second variable"
      },
      {
        "name": "Variable c",
        "nickname": "c",
        "access": "item",
        "description": "The third variable"
      },
      {
        "name": "Variable x",
        "nickname": "x",
        "access": "item",
        "description": "The fourth variable"
      },
      {
        "name": "Variable y",
        "nickname": "y",
        "access": "item",
        "description": "The fifth variable"
      },
      {
        "name": "Variable z",
        "nickname": "z",
        "access": "item",
        "description": "The sixth variable"
      }
    ],
    "outputs": [
      {
        "name": "Value",
        "nickname": "V",
        "access": "item",
        "description": "Expression result"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Domain",
    "name": "Divide Domain²",
    "nickname": "Divide",
    "guid": "75ac008b-1bc2-4edd-b967-667d628b9d24",
    "description": "Divides a two-dimensional domain into equal segments.",
    "inputs": [
      {
        "name": "Domain",
        "nickname": "I",
        "access": "item",
        "description": "Base domain"
      },
      {
        "name": "U Count",
        "nickname": "U",
        "access": "item",
        "description": "Number of segments in {u} direction"
      },
      {
        "name": "V Count",
        "nickname": "V",
        "access": "item",
        "description": "Number of segments in {v} direction"
      }
    ],
    "outputs": [
      {
        "name": "Segments",
        "nickname": "S",
        "access": "list",
        "description": "Individual segments"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Domain",
    "name": "Divide Domain",
    "nickname": "Div",
    "guid": "75ef4190-91a2-42d9-a245-32a7162b0384",
    "description": "Divide a domain into equal segments.",
    "inputs": [
      {
        "name": "Domain",
        "nickname": "I",
        "access": "item",
        "description": "Base domain"
      },
      {
        "name": "Count",
        "nickname": "C",
        "access": "item",
        "description": "Number of segments"
      }
    ],
    "outputs": [
      {
        "name": "Segments",
        "nickname": "S",
        "access": "list",
        "description": "Division segments"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Trig",
    "name": "Sine",
    "nickname": "Sin",
    "guid": "7663efbb-d9b8-4c6a-a0da-c3750a7bbe77",
    "description": "Compute the sine of a value",
    "inputs": [
      {
        "name": "Value",
        "nickname": "x",
        "access": "item",
        "description": "Input value"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "y",
        "access": "item",
        "description": "Output value"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Operators",
    "name": "Gate Majority",
    "nickname": "Vote",
    "guid": "78669f9c-4fea-44fd-ab12-2a69eeec58de",
    "description": "Calculates the majority vote among three booleans.",
    "inputs": [
      {
        "name": "A",
        "nickname": "A",
        "access": "item",
        "description": "First boolean"
      },
      {
        "name": "B",
        "nickname": "B",
        "access": "item",
        "description": "Second boolean"
      },
      {
        "name": "C",
        "nickname": "C",
        "access": "item",
        "description": "Third boolean"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "R",
        "access": "item",
        "description": "Average value"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Operators",
    "name": "Power",
    "nickname": "Pow",
    "guid": "78fed580-851b-46fe-af2f-6519a9d378e0",
    "description": "Raise a value to a power.",
    "inputs": [
      {
        "name": "A",
        "nickname": "A",
        "access": "item",
        "description": "The item to be raised"
      },
      {
        "name": "B",
        "nickname": "B",
        "access": "item",
        "description": "The exponent"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "R",
        "access": "item",
        "description": "A raised to the B power"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Polynomials",
    "name": "One Over X",
    "nickname": "1/x",
    "guid": "797d922f-3a1d-46fe-9155-358b009b5997",
    "description": "Compute one over x.",
    "inputs": [
      {
        "name": "Value",
        "nickname": "x",
        "access": "item",
        "description": "Input value"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "y",
        "access": "item",
        "description": "Output value"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Util",
    "name": "Average",
    "nickname": "Avr",
    "guid": "7986486c-621a-48fb-8f27-a28a22c91cc9",
    "description": "Solve the arithmetic average for a set of items",
    "inputs": [
      {
        "name": "Input",
        "nickname": "I",
        "access": "list",
        "description": "Input values for averaging"
      }
    ],
    "outputs": [
      {
        "name": "Arithmetic mean",
        "nickname": "AM",
        "access": "item",
        "description": "Arithmetic mean (average) of all input values"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Polynomials",
    "name": "Power of 2",
    "nickname": "2º",
    "guid": "7a1e5fd7-b7da-4244-a261-f1da66614992",
    "description": "Raise 2 to the power of N.",
    "inputs": [
      {
        "name": "Value",
        "nickname": "x",
        "access": "item",
        "description": "Input value"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "y",
        "access": "item",
        "description": "Output value"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Polynomials",
    "name": "Log N",
    "nickname": "LogN",
    "guid": "7ab8d289-26a2-4dd4-b4ad-df5b477999d8",
    "description": "Return the N-base logarithm of a number.",
    "inputs": [
      {
        "name": "Number",
        "nickname": "V",
        "access": "item",
        "description": "Value"
      },
      {
        "name": "Base",
        "nickname": "B",
        "access": "item",
        "description": "Logarithm base"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "R",
        "access": "item",
        "description": "Result"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Util",
    "name": "Complex Conjugate",
    "nickname": "z*",
    "guid": "7d2a6064-51f0-45b2-adc4-f417b30dcd15",
    "description": "Create the conjugate of a Complex number",
    "inputs": [
      {
        "name": "Complex",
        "nickname": "C",
        "access": "item",
        "description": "Complex number"
      }
    ],
    "outputs": [
      {
        "name": "Conjugate",
        "nickname": "C",
        "access": "item",
        "description": "Conjugate of the Complex number [C]"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Polynomials",
    "name": "Cube",
    "nickname": "Cube",
    "guid": "7e3185eb-a38c-4949-bcf2-0e80dee3a344",
    "description": "Compute the cube of a value",
    "inputs": [
      {
        "name": "Value",
        "nickname": "x",
        "access": "item",
        "description": "Input value"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "y",
        "access": "item",
        "description": "Output value"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Operators",
    "name": "Factorial",
    "nickname": "Fac",
    "guid": "80da90e3-3ea9-4cfe-b7cc-2b6019f850e3",
    "description": "Returns the factorial of an integer.",
    "inputs": [
      {
        "name": "Number",
        "nickname": "N",
        "access": "item",
        "description": "Input integer"
      }
    ],
    "outputs": [
      {
        "name": "Factorial",
        "nickname": "F",
        "access": "item",
        "description": "Factorial of {N}"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Domain",
    "name": "Deconstruct Domain",
    "nickname": "DeDomain",
    "guid": "825ea536-aebb-41e9-af32-8baeb2ecb590",
    "description": "Deconstruct a numeric domain into its component parts.",
    "inputs": [
      {
        "name": "Domain",
        "nickname": "I",
        "access": "item",
        "description": "Base domain"
      }
    ],
    "outputs": [
      {
        "name": "Start",
        "nickname": "S",
        "access": "item",
        "description": "Start of domain"
      },
      {
        "name": "End",
        "nickname": "E",
        "access": "item",
        "description": "End of domain"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Domain",
    "name": "Construct Domain²",
    "nickname": "Dom²",
    "guid": "8555a743-36c1-42b8-abcc-06d9cb94519f",
    "description": "Create a two-dimensional domain from two simple domains.",
    "inputs": [
      {
        "name": "Domain U",
        "nickname": "U",
        "access": "item",
        "description": "Domain in {u} direction"
      },
      {
        "name": "Domain V",
        "nickname": "V",
        "access": "item",
        "description": "Domain in {v} direction"
      }
    ],
    "outputs": [
      {
        "name": "2D Domain",
        "nickname": "I²",
        "access": "item",
        "description": "Two dimensional numeric domain of {u} and {v}"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Matrix",
    "name": "Swap Rows",
    "nickname": "SwapR",
    "guid": "8600a3fc-30f0-4df6-b126-aaa79ece5bfe",
    "description": "Swap two rows in a matrix",
    "inputs": [
      {
        "name": "Matrix",
        "nickname": "M",
        "access": "item",
        "description": "Matrix for row swap"
      },
      {
        "name": "Row A",
        "nickname": "A",
        "access": "item",
        "description": "First row index"
      },
      {
        "name": "Row B",
        "nickname": "B",
        "access": "item",
        "description": "Second row index"
      }
    ],
    "outputs": [
      {
        "name": "Matrix",
        "nickname": "M",
        "access": "item",
        "description": "Matrix with swapped rows"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Script",
    "name": "DotNET C# Script (LEGACY)",
    "nickname": "C#",
    "guid": "88c3f2b5-27f7-48a2-9528-1397fad62b93",
    "description": "A C#.NET scriptable component",
    "inputs": [
      {
        "name": "x",
        "nickname": "x",
        "access": "item",
        "description": "Script Variable x"
      },
      {
        "name": "y",
        "nickname": "y",
        "access": "item",
        "description": "Script Variable y"
      }
    ],
    "outputs": [
      {
        "name": "Output",
        "nickname": "out",
        "access": "item",
        "description": "Print, Reflect and Error streams"
      },
      {
        "name": "A",
        "nickname": "A",
        "access": "item",
        "description": "Output parameter A"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Util",
    "name": "Complex Modulus",
    "nickname": "CMod",
    "guid": "88fb33f9-f467-452b-a0e3-44bdb78a9b06",
    "description": "Get the modulus of a Complex number",
    "inputs": [
      {
        "name": "Complex",
        "nickname": "C",
        "access": "item",
        "description": "Complex number"
      }
    ],
    "outputs": [
      {
        "name": "Modulus",
        "nickname": "M",
        "access": "item",
        "description": "Modulus of the Complex number [C]"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Domain",
    "name": "Construct Domain²",
    "nickname": "Dom²Num",
    "guid": "9083b87f-a98c-4e41-9591-077ae4220b19",
    "description": "Create a two-dimensinal domain from four numbers.",
    "inputs": [
      {
        "name": "U min",
        "nickname": "U0",
        "access": "item",
        "description": "Lower limit of domain in {u} direction"
      },
      {
        "name": "U max",
        "nickname": "U1",
        "access": "item",
        "description": "Upper limit of domain in {u} direction"
      },
      {
        "name": "V min",
        "nickname": "V0",
        "access": "item",
        "description": "Lower limit of domain in {v} direction"
      },
      {
        "name": "V max",
        "nickname": "V1",
        "access": "item",
        "description": "Upper limit of domain in {v} direction"
      }
    ],
    "outputs": [
      {
        "name": "2D Domain",
        "nickname": "I²",
        "access": "item",
        "description": "Two dimensional numeric domain of {u} and {v}"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Operators",
    "name": "Mass Multiplication",
    "nickname": "MM",
    "guid": "921775f7-bf22-4cfc-a4db-c415a56069c4",
    "description": "Perform mass multiplication of a list of numbers",
    "inputs": [
      {
        "name": "Input",
        "nickname": "I",
        "access": "list",
        "description": "Input numbers for mass multiplication"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "R",
        "access": "item",
        "description": "Result of mass multiplication"
      },
      {
        "name": "Partial Results",
        "nickname": "Pr",
        "access": "list",
        "description": "List of partial results"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Trig",
    "name": "Triangle Trigonometry",
    "nickname": "Trig",
    "guid": "92af1a02-9b87-43a0-8c45-0ce1b81555ec",
    "description": "Generic triangle trigonometry",
    "inputs": [
      {
        "name": "Alpha",
        "nickname": "α",
        "access": "item",
        "description": "Optional alpha angle"
      },
      {
        "name": "Beta",
        "nickname": "β",
        "access": "item",
        "description": "Optional beta angle"
      },
      {
        "name": "Gamma",
        "nickname": "γ",
        "access": "item",
        "description": "Optional gamma angle"
      },
      {
        "name": "A length",
        "nickname": "A",
        "access": "item",
        "description": "Optional length of A edge (opposite alpha)"
      },
      {
        "name": "B length",
        "nickname": "B",
        "access": "item",
        "description": "Optional length of B edge (opposite beta)"
      },
      {
        "name": "C length",
        "nickname": "C",
        "access": "item",
        "description": "Optional length of C edge (opposite gamma)"
      }
    ],
    "outputs": [
      {
        "name": "Alpha",
        "nickname": "α",
        "access": "item",
        "description": "Computed alpha angle"
      },
      {
        "name": "Beta",
        "nickname": "β",
        "access": "item",
        "description": "Computed beta angle"
      },
      {
        "name": "Gamma",
        "nickname": "γ",
        "access": "item",
        "description": "Computed gamma angle"
      },
      {
        "name": "A length",
        "nickname": "A",
        "access": "item",
        "description": "Computed length of A edge"
      },
      {
        "name": "B length",
        "nickname": "B",
        "access": "item",
        "description": "Computed length of B edge"
      },
      {
        "name": "C length",
        "nickname": "C",
        "access": "item",
        "description": "Computed length of C edge"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Domain",
    "name": "Consecutive Domains",
    "nickname": "Consec",
    "guid": "95992b33-89e1-4d36-bd35-2754a11af21e",
    "description": "Create consecutive domains from a list of numbers",
    "inputs": [
      {
        "name": "Numbers",
        "nickname": "N",
        "access": "list",
        "description": "Numbers for consecutive domains"
      },
      {
        "name": "Additive",
        "nickname": "A",
        "access": "item",
        "description": "If True, values are added to a sum-total"
      }
    ],
    "outputs": [
      {
        "name": "Domains",
        "nickname": "D",
        "access": "list",
        "description": "Domains describing the spaces between the numbers"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Domain",
    "name": "Remap Numbers",
    "nickname": "ReMap",
    "guid": "9624aeeb-f2a1-49da-b1c7-8789db217177",
    "description": "Remap numbers into a new numeric domain",
    "inputs": [
      {
        "name": "Values",
        "nickname": "V",
        "access": "list",
        "description": "Values to remap"
      },
      {
        "name": "Source",
        "nickname": "S",
        "access": "item",
        "description": "Optional source domain. If left blank, the value range will be used."
      },
      {
        "name": "Target",
        "nickname": "T",
        "access": "item",
        "description": "Target domain"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "R",
        "access": "list",
        "description": "Remapped numbers."
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Operators",
    "name": "Subtraction",
    "nickname": "A-B",
    "guid": "9c007a04-d0d9-48e4-9da3-9ba142bc4d46",
    "description": "Mathematical subtraction",
    "inputs": [
      {
        "name": "A",
        "nickname": "A",
        "access": "item",
        "description": "First operand for subtraction"
      },
      {
        "name": "B",
        "nickname": "B",
        "access": "item",
        "description": "Second operand for subtraction"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "R",
        "access": "item",
        "description": "Result of subtraction"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Operators",
    "name": "Division",
    "nickname": "A/B",
    "guid": "9c85271f-89fa-4e9f-9f4a-d75802120ccc",
    "description": "Mathematical division",
    "inputs": [
      {
        "name": "A",
        "nickname": "A",
        "access": "item",
        "description": "Item to divide (dividend)"
      },
      {
        "name": "B",
        "nickname": "B",
        "access": "item",
        "description": "Item to divide with (divisor)"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "R",
        "access": "item",
        "description": "The result of the Division"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Script",
    "name": "Expression",
    "nickname": "Expression",
    "guid": "9df5e896-552d-4c8c-b9ca-4fc147ffa022",
    "description": "Evaluate an expression",
    "inputs": [
      {
        "name": "Variable x",
        "nickname": "x",
        "access": "item",
        "description": "Expression variable"
      },
      {
        "name": "Variable y",
        "nickname": "y",
        "access": "item",
        "description": "Expression variable"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "R",
        "access": "item",
        "description": "Result of expression"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Operators",
    "name": "Factorial",
    "nickname": "Fac",
    "guid": "a0a38131-c5fc-4984-b05d-34cf57f0c018",
    "description": "Returns the factorial of an integer.",
    "inputs": [
      {
        "name": "Number",
        "nickname": "N",
        "access": "item",
        "description": "Input integer"
      }
    ],
    "outputs": [
      {
        "name": "Factorial",
        "nickname": "F",
        "access": "item",
        "description": "Factorial of {N}"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Operators",
    "name": "Addition",
    "nickname": "A+B",
    "guid": "a0d62394-a118-422d-abb3-6af115c75b25",
    "description": "Mathematical addition",
    "inputs": [
      {
        "name": "A",
        "nickname": "A",
        "access": "item",
        "description": "First item for addition"
      },
      {
        "name": "B",
        "nickname": "B",
        "access": "item",
        "description": "Second item for addition"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "R",
        "access": "item",
        "description": "Result of addition"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Trig",
    "name": "Sinc",
    "nickname": "Sinc",
    "guid": "a2d9503d-a83c-4d71-81e0-02af8d09cd0c",
    "description": "Compute the sinc (Sinus Cardinalis) of a value.",
    "inputs": [
      {
        "name": "Value",
        "nickname": "x",
        "access": "item",
        "description": "Input value"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "y",
        "access": "item",
        "description": "Output value"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Operators",
    "name": "Negative",
    "nickname": "Neg",
    "guid": "a3371040-e552-4bc8-b0ff-10a840258e88",
    "description": "Compute the negative of a value.",
    "inputs": [
      {
        "name": "Value",
        "nickname": "x",
        "access": "item",
        "description": "Input value"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "y",
        "access": "item",
        "description": "Output value"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Trig",
    "name": "Radians",
    "nickname": "Rad",
    "guid": "a4cd2751-414d-42ec-8916-476ebf62d7fe",
    "description": "Convert an angle specified in degrees to radians",
    "inputs": [
      {
        "name": "Degrees",
        "nickname": "D",
        "access": "item",
        "description": "Angle in degrees"
      }
    ],
    "outputs": [
      {
        "name": "Radians",
        "nickname": "R",
        "access": "item",
        "description": "Angle in radians"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Util",
    "name": "Round",
    "nickname": "Round",
    "guid": "a50c4a3b-0177-4c91-8556-db95de6c56c8",
    "description": "Round a floating point value.",
    "inputs": [
      {
        "name": "Number",
        "nickname": "x",
        "access": "item",
        "description": "Number to round"
      }
    ],
    "outputs": [
      {
        "name": "Nearest",
        "nickname": "N",
        "access": "item",
        "description": "Integer nearest to x"
      },
      {
        "name": "Floor",
        "nickname": "F",
        "access": "item",
        "description": "First integer smaller than or equal to x"
      },
      {
        "name": "Ceiling",
        "nickname": "C",
        "access": "item",
        "description": "First integer larger than or equal to x"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Script",
    "name": "C# Script",
    "nickname": "C#",
    "guid": "a9a8ebd2-fff5-4c44-a8f5-739736d129ba",
    "description": "A C#.NET scriptable component",
    "inputs": [
      {
        "name": "x",
        "nickname": "x",
        "access": "item",
        "description": "Script Variable x"
      },
      {
        "name": "y",
        "nickname": "y",
        "access": "item",
        "description": "Script Variable y"
      }
    ],
    "outputs": [
      {
        "name": "out",
        "nickname": "out",
        "access": "list",
        "description": "Print, Reflect and Error streams"
      },
      {
        "name": "A",
        "nickname": "A",
        "access": "item",
        "description": "Output parameter A"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Polynomials",
    "name": "Square Root",
    "nickname": "Sqrt",
    "guid": "ad476cb7-b6d1-41c8-986b-0df243a64146",
    "description": "Compute the square root of a value",
    "inputs": [
      {
        "name": "Value",
        "nickname": "x",
        "access": "item",
        "description": "Input value"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "y",
        "access": "item",
        "description": "Output value"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Operators",
    "name": "Smaller Than",
    "nickname": "Smaller",
    "guid": "ae840986-cade-4e5a-96b0-570f007d4fc0",
    "description": "Smaller than (or equal to)",
    "inputs": [
      {
        "name": "First Number",
        "nickname": "A",
        "access": "item",
        "description": "Number to test"
      },
      {
        "name": "Second Number",
        "nickname": "B",
        "access": "item",
        "description": "Number to test against"
      }
    ],
    "outputs": [
      {
        "name": "Smaller than",
        "nickname": "<",
        "access": "item",
        "description": "True if A < B"
      },
      {
        "name": "… or Equal to",
        "nickname": "<=",
        "access": "item",
        "description": "True if A <= B"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Trig",
    "name": "Centroid",
    "nickname": "Centroid",
    "guid": "afbcbad4-2a2a-4954-8040-d999e316d2bd",
    "description": "Generate the triangle centroid from medians.",
    "inputs": [
      {
        "name": "Point A",
        "nickname": "A",
        "access": "item",
        "description": "First triangle corner"
      },
      {
        "name": "Point B",
        "nickname": "B",
        "access": "item",
        "description": "Second triangle corner"
      },
      {
        "name": "Point C",
        "nickname": "C",
        "access": "item",
        "description": "Third triangle corner"
      }
    ],
    "outputs": [
      {
        "name": "Centroid",
        "nickname": "C",
        "access": "item",
        "description": "Centroid point for triangle"
      },
      {
        "name": "Median AB",
        "nickname": "AB",
        "access": "item",
        "description": "Median line connecting edge AB with corner C"
      },
      {
        "name": "Median BC",
        "nickname": "BC",
        "access": "item",
        "description": "Median line connecting edge BC with corner A"
      },
      {
        "name": "Median CA",
        "nickname": "CA",
        "access": "item",
        "description": "Median line connecting edge CA with corner B"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Trig",
    "name": "ArcTangent",
    "nickname": "ATan",
    "guid": "b4647919-d041-419e-99f5-fa0dc0ddb8b6",
    "description": "Compute the angle whose tangent is the specified value.",
    "inputs": [
      {
        "name": "Value",
        "nickname": "x",
        "access": "item",
        "description": "Input value"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "y",
        "access": "item",
        "description": "Output value"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Operators",
    "name": "Gate Xnor",
    "nickname": "Xnor",
    "guid": "b6aedcac-bf43-42d4-899e-d763612f834d",
    "description": "Perform boolean biconditional (XNOR gate).",
    "inputs": [
      {
        "name": "A",
        "nickname": "A",
        "access": "item",
        "description": "Left hand boolean"
      },
      {
        "name": "B",
        "nickname": "B",
        "access": "item",
        "description": "Right hand boolean"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "R",
        "access": "item",
        "description": "Resulting value"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Util",
    "name": "Natural logarithm",
    "nickname": "E",
    "guid": "b6cac37c-21b9-46c6-bd0d-17ff67796578",
    "description": "Returns a factor of the natural number (e).",
    "inputs": [
      {
        "name": "Factor",
        "nickname": "N",
        "access": "item",
        "description": "Factor to be multiplied by e"
      }
    ],
    "outputs": [
      {
        "name": "Output",
        "nickname": "y",
        "access": "item",
        "description": "Output value"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Operators",
    "name": "Multiplication",
    "nickname": "A×B",
    "guid": "b8963bb1-aa57-476e-a20e-ed6cf635a49c",
    "description": "Mathematical multiplication",
    "inputs": [
      {
        "name": "A",
        "nickname": "A",
        "access": "item",
        "description": "First item for multiplication"
      },
      {
        "name": "B",
        "nickname": "B",
        "access": "item",
        "description": "Second item for multiplication"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "R",
        "access": "item",
        "description": "The result of the Multiplication"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Util",
    "name": "Truncate",
    "nickname": "Trunc",
    "guid": "bd96f893-d57b-4f04-90d0-dca0d72ff2f9",
    "description": "Perform truncation of numerical extremes",
    "inputs": [
      {
        "name": "Input",
        "nickname": "I",
        "access": "list",
        "description": "Input values for truncation"
      },
      {
        "name": "Truncation factor",
        "nickname": "t",
        "access": "item",
        "description": "Truncation factor. Must be between 0.0 (no trucation) and 1.0 (full truncation)"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "T",
        "access": "list",
        "description": "Truncated set"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Util",
    "name": "Complex Argument",
    "nickname": "Arg",
    "guid": "be715e4c-d6d8-447b-a9c3-6fea700d0b83",
    "description": "Get the argument of a Complex number",
    "inputs": [
      {
        "name": "Complex",
        "nickname": "C",
        "access": "item",
        "description": "Complex number"
      }
    ],
    "outputs": [
      {
        "name": "Argument",
        "nickname": "A",
        "access": "item",
        "description": "Argument of the Complex number [C]"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Boolean",
    "name": "Gate And Ternary",
    "nickname": "And",
    "guid": "c1364962-87dd-4a6d-901a-e5b170e5ef9e",
    "description": "Perform ternary boolean conjunction (AND gate).",
    "inputs": [
      {
        "name": "A",
        "nickname": "A",
        "access": "item",
        "description": "First boolean"
      },
      {
        "name": "B",
        "nickname": "B",
        "access": "item",
        "description": "Second boolean"
      },
      {
        "name": "C",
        "nickname": "C",
        "access": "item",
        "description": "Third boolean"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "R",
        "access": "item",
        "description": "Resulting value"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Trig",
    "name": "Incentre",
    "nickname": "ICentre",
    "guid": "c3342ea2-e181-46aa-a9b9-e438ccbfb831",
    "description": "Generate the triangle incentre from angle bisectors.",
    "inputs": [
      {
        "name": "Point A",
        "nickname": "A",
        "access": "item",
        "description": "First triangle corner"
      },
      {
        "name": "Point B",
        "nickname": "B",
        "access": "item",
        "description": "Second triangle corner"
      },
      {
        "name": "Point C",
        "nickname": "C",
        "access": "item",
        "description": "Third triangle corner"
      }
    ],
    "outputs": [
      {
        "name": "Incentre",
        "nickname": "I",
        "access": "item",
        "description": "Incentre point for triangle"
      },
      {
        "name": "Bisector A",
        "nickname": "A",
        "access": "item",
        "description": "Perpendicular bisector line emanating from corner A"
      },
      {
        "name": "Bisector B",
        "nickname": "B",
        "access": "item",
        "description": "Perpendicular bisector line emanating from corner B"
      },
      {
        "name": "Bisector C",
        "nickname": "C",
        "access": "item",
        "description": "Perpendicular bisector line emanating from corner C"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Polynomials",
    "name": "Power of E",
    "nickname": "Eº",
    "guid": "c717f26f-e4a0-475c-8e1c-b8f77af1bc99",
    "description": "Raise E to the power of N.",
    "inputs": [
      {
        "name": "Value",
        "nickname": "x",
        "access": "item",
        "description": "Input value"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "y",
        "access": "item",
        "description": "Output value"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Util",
    "name": "Golden Ratio",
    "nickname": "Phi",
    "guid": "cb22d3ed-93d8-4629-bdf2-c0c7c25afd2c",
    "description": "Returns a factor of the golden ratio (Phi).",
    "inputs": [
      {
        "name": "Factor",
        "nickname": "N",
        "access": "item",
        "description": "Factor to be multiplied by Phi"
      }
    ],
    "outputs": [
      {
        "name": "Output",
        "nickname": "y",
        "access": "item",
        "description": "Output value"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Operators",
    "name": "Gate Not",
    "nickname": "Not",
    "guid": "cb2c7d3c-41b4-4c6d-a6bd-9235bd2851bb",
    "description": "Perform boolean negation (NOT gate).",
    "inputs": [
      {
        "name": "A",
        "nickname": "A",
        "access": "item",
        "description": "Boolean value"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "R",
        "access": "item",
        "description": "Inverse of {A}"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Trig",
    "name": "ArcSine",
    "nickname": "ASin",
    "guid": "cc15ba56-fae7-4f05-b599-cb7c43b60e11",
    "description": "Compute the angle whose sine is the specified value.",
    "inputs": [
      {
        "name": "Value",
        "nickname": "x",
        "access": "item",
        "description": "Input value"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "y",
        "access": "item",
        "description": "Output value"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Script",
    "name": "Evaluate",
    "nickname": "Eval",
    "guid": "cc2b626f-6eff-4d08-9829-2877560693f4",
    "description": "Evaluate an expression with a flexible number of variables.",
    "inputs": [
      {
        "name": "Expression",
        "nickname": "F",
        "access": "item",
        "description": "Expression to evaluate"
      },
      {
        "name": "Variable x",
        "nickname": "x",
        "access": "item",
        "description": "Expression variable"
      },
      {
        "name": "Variable y",
        "nickname": "y",
        "access": "item",
        "description": "Expression variable"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "r",
        "access": "item",
        "description": "Expression result"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Operators",
    "name": "Multiplication",
    "nickname": "A×B",
    "guid": "ce46b74e-00c9-43c4-805a-193b69ea4a11",
    "description": "Mathematical multiplication",
    "inputs": [
      {
        "name": "A",
        "nickname": "A",
        "access": "item",
        "description": "First item for multiplication"
      },
      {
        "name": "B",
        "nickname": "B",
        "access": "item",
        "description": "Second item for multiplication"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "R",
        "access": "item",
        "description": "Result of multiplication"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Script",
    "name": "GhPython Script",
    "nickname": "Python",
    "guid": "ceab6e56-ceec-a646-84d5-363c57440969",
    "description": "GhPython provides a Python script component",
    "inputs": [
      {
        "name": "x",
        "nickname": "x",
        "access": "item",
        "description": "x"
      },
      {
        "name": "y",
        "nickname": "y",
        "access": "item",
        "description": "y"
      }
    ],
    "outputs": [
      {
        "name": "out",
        "nickname": "out",
        "access": "item",
        "description": "The execution information, as output and error streams"
      },
      {
        "name": "Result a",
        "nickname": "a",
        "access": "item",
        "description": "Output parameter a"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Operators",
    "name": "Addition",
    "nickname": "A+B",
    "guid": "d18db32b-7099-4eea-85c4-8ba675ee8ec3",
    "description": "Mathematical addition",
    "inputs": [
      {
        "name": "A",
        "nickname": "A",
        "access": "item",
        "description": "First item for addition"
      },
      {
        "name": "B",
        "nickname": "B",
        "access": "item",
        "description": "Second item for addition"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "R",
        "access": "item",
        "description": "The result of the Addition"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Domain",
    "name": "Construct Domain",
    "nickname": "Dom",
    "guid": "d1a28e95-cf96-4936-bf34-8bf142d731bf",
    "description": "Create a numeric domain from two numeric extremes.",
    "inputs": [
      {
        "name": "Domain start",
        "nickname": "A",
        "access": "item",
        "description": "Start value of numeric domain"
      },
      {
        "name": "Domain end",
        "nickname": "B",
        "access": "item",
        "description": "End value of numeric domain"
      }
    ],
    "outputs": [
      {
        "name": "Domain",
        "nickname": "I",
        "access": "item",
        "description": "Numeric domain between {A} and {B}"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Trig",
    "name": "CoSecant",
    "nickname": "Csc",
    "guid": "d222500b-dfd5-45e0-933e-eabefd07cbfa",
    "description": "Compute the co-secant (reciprocal of the Sine) of an angle.",
    "inputs": [
      {
        "name": "Value",
        "nickname": "x",
        "access": "item",
        "description": "Input value"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "y",
        "access": "item",
        "description": "Output value"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Script",
    "name": "F(x,y) [OBSOLETE]",
    "nickname": "F(x,y)",
    "guid": "d2b10b82-f612-4763-91ca-0cbdbe276171",
    "description": "A function of two variables x and y",
    "inputs": [
      {
        "name": "Function",
        "nickname": "F",
        "access": "item",
        "description": "The function script"
      },
      {
        "name": "Variable x",
        "nickname": "x",
        "access": "item",
        "description": "The first variable"
      },
      {
        "name": "Variable y",
        "nickname": "y",
        "access": "item",
        "description": "The second variable"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "r",
        "access": "item",
        "description": "Expression result"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Trig",
    "name": "Cosine",
    "nickname": "Cos",
    "guid": "d2d2a900-780c-4d58-9a35-1f9d8d35df6f",
    "description": "Compute the cosine of a value",
    "inputs": [
      {
        "name": "Value",
        "nickname": "x",
        "access": "item",
        "description": "Input value"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "y",
        "access": "item",
        "description": "Output value"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Script",
    "name": "F(x) [OBSOLETE]",
    "nickname": "F(x)",
    "guid": "d3e721b4-f5ea-4e40-85fc-b68616939e47",
    "description": "A function of a single variable x (OBSOLETE)",
    "inputs": [
      {
        "name": "Function",
        "nickname": "F",
        "access": "item",
        "description": "The function script"
      },
      {
        "name": "Variable X",
        "nickname": "x",
        "access": "item",
        "description": "The variable to solve"
      }
    ],
    "outputs": [
      {
        "name": "Result Y",
        "nickname": "y",
        "access": "item",
        "description": "Equation solution"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Time",
    "name": "Deconstruct Date",
    "nickname": "DDate",
    "guid": "d5e28df8-495b-4892-bca8-60748743d955",
    "description": "Deconstruct a date into years, months, days, hours, minutes and seconds",
    "inputs": [
      {
        "name": "Date",
        "nickname": "D",
        "access": "item",
        "description": "Date and Time data"
      }
    ],
    "outputs": [
      {
        "name": "Year",
        "nickname": "Y",
        "access": "item",
        "description": "Year number"
      },
      {
        "name": "Month",
        "nickname": "M",
        "access": "item",
        "description": "Month number"
      },
      {
        "name": "Day",
        "nickname": "D",
        "access": "item",
        "description": "Day of month"
      },
      {
        "name": "Hour",
        "nickname": "h",
        "access": "item",
        "description": "Hour of day"
      },
      {
        "name": "Minute",
        "nickname": "m",
        "access": "item",
        "description": "Minute of the hour"
      },
      {
        "name": "Second",
        "nickname": "s",
        "access": "item",
        "description": "Second of the minute"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Operators",
    "name": "Relative Differences",
    "nickname": "RelDif",
    "guid": "dd17d442-3776-40b3-ad5b-5e188b56bd4c",
    "description": "Compute relative differences for a list of data",
    "inputs": [
      {
        "name": "Values",
        "nickname": "V",
        "access": "list",
        "description": "List of data to operate on (numbers or points or vectors allowed)"
      }
    ],
    "outputs": [
      {
        "name": "Differenced",
        "nickname": "D",
        "access": "list",
        "description": "Differences between consecutive items"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Domain",
    "name": "Bounds 2D",
    "nickname": "Bnd",
    "guid": "dd53b24c-003a-4a04-b185-a44d91633cbe",
    "description": "Create a numeric two-dimensional domain which encompasses a list of coordinates.",
    "inputs": [
      {
        "name": "Coordinates",
        "nickname": "C",
        "access": "list",
        "description": "Two dimensional coordinates to include in Bounds"
      }
    ],
    "outputs": [
      {
        "name": "Domain",
        "nickname": "I",
        "access": "item",
        "description": "Numeric two-dimensional domain between the lowest and highest numbers in {N.x ; N.y}"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Operators",
    "name": "Gate Xor",
    "nickname": "Xor",
    "guid": "de4a0d86-2709-4564-935a-88bf4d40af89",
    "description": "Perform boolean exclusive disjunction (XOR gate).",
    "inputs": [
      {
        "name": "A",
        "nickname": "A",
        "access": "item",
        "description": "Left hand boolean"
      },
      {
        "name": "B",
        "nickname": "B",
        "access": "item",
        "description": "Right hand boolean"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "R",
        "access": "item",
        "description": "Resulting value"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Util",
    "name": "Epsilon",
    "nickname": "Eps",
    "guid": "deadf87d-99a6-4980-90c3-f98350aa6f0f",
    "description": "Returns a factor of double precision floating point epsilon.",
    "inputs": [
      {
        "name": "Factor",
        "nickname": "N",
        "access": "item",
        "description": "Factor to be multiplied by epsilon"
      }
    ],
    "outputs": [
      {
        "name": "Output",
        "nickname": "y",
        "access": "item",
        "description": "Output value"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Util",
    "name": "Interpolate data",
    "nickname": "Interp",
    "guid": "e168ff6b-e5c0-48f1-b831-f6996bf3b459",
    "description": "Interpolate a collection of data.",
    "inputs": [
      {
        "name": "Data",
        "nickname": "D",
        "access": "list",
        "description": "Data to interpolate (simple data types only)."
      },
      {
        "name": "Parameter",
        "nickname": "t",
        "access": "item",
        "description": "Normalised interpolation parameter."
      }
    ],
    "outputs": [
      {
        "name": "Value",
        "nickname": "V",
        "access": "item",
        "description": "Interpolated value."
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Script",
    "name": "F(x,y,z) [OBSOLETE]",
    "nickname": "F(x,y,z)",
    "guid": "e1c4bccc-4ecf-4f18-885d-dfd8983e572a",
    "description": "A function of three variables x, y and z",
    "inputs": [
      {
        "name": "Function",
        "nickname": "F",
        "access": "item",
        "description": "The function script"
      },
      {
        "name": "Variable x",
        "nickname": "x",
        "access": "item",
        "description": "The first variable"
      },
      {
        "name": "Variable y",
        "nickname": "y",
        "access": "item",
        "description": "The second variable"
      },
      {
        "name": "Variable z",
        "nickname": "z",
        "access": "item",
        "description": "The third variable"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "r",
        "access": "item",
        "description": "Expression result"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Operators",
    "name": "Mass Multiplication",
    "nickname": "MM",
    "guid": "e44c1bd7-72cc-4697-80c9-02787baf7bb4",
    "description": "Perform mass multiplication of a list of items",
    "inputs": [
      {
        "name": "Input",
        "nickname": "I",
        "access": "list",
        "description": "Input values for mass multiplication."
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "R",
        "access": "item",
        "description": "Result of mass multiplication"
      },
      {
        "name": "Partial Results",
        "nickname": "Pr",
        "access": "list",
        "description": "List of partial results"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Time",
    "name": "Construct Exotic Date",
    "nickname": "DateEx",
    "guid": "e5ff52c5-40df-4f43-ac3b-d2418d05ae32",
    "description": "Construct a date using a specific calendar",
    "inputs": [
      {
        "name": "Year",
        "nickname": "Y",
        "access": "item",
        "description": "Year number (must be between 1 and 9999)"
      },
      {
        "name": "Month",
        "nickname": "M",
        "access": "item",
        "description": "Month number (must be between 1 and 12)"
      },
      {
        "name": "Day",
        "nickname": "D",
        "access": "item",
        "description": "Day of month (must be between 1 and 31)"
      }
    ],
    "outputs": [
      {
        "name": "Time",
        "nickname": "T",
        "access": "item",
        "description": "Gregorian representation of date."
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Trig",
    "name": "Right Trigonometry",
    "nickname": "RTrig",
    "guid": "e75d4624-8ee2-4067-ac8d-c56bdc901d83",
    "description": "Right triangle trigonometry",
    "inputs": [
      {
        "name": "Alpha",
        "nickname": "α",
        "access": "item",
        "description": "Optional alpha angle"
      },
      {
        "name": "Beta",
        "nickname": "β",
        "access": "item",
        "description": "Optional beta angle"
      },
      {
        "name": "P length",
        "nickname": "P",
        "access": "item",
        "description": "Optional length of P edge"
      },
      {
        "name": "Q length",
        "nickname": "Q",
        "access": "item",
        "description": "Optional length of Q edge"
      },
      {
        "name": "R length",
        "nickname": "R",
        "access": "item",
        "description": "Optional length of R edge"
      }
    ],
    "outputs": [
      {
        "name": "Alpha",
        "nickname": "α",
        "access": "item",
        "description": "Computed alpha angle"
      },
      {
        "name": "Beta",
        "nickname": "β",
        "access": "item",
        "description": "Computed beta angle"
      },
      {
        "name": "P length",
        "nickname": "P",
        "access": "item",
        "description": "Computed length of P edge"
      },
      {
        "name": "Q length",
        "nickname": "Q",
        "access": "item",
        "description": "Computed length of Q edge"
      },
      {
        "name": "R length",
        "nickname": "R",
        "access": "item",
        "description": "Computed length of R edge"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Script",
    "name": "F7",
    "nickname": "F(a,b,c,d,x,y,z)",
    "guid": "e9628b21-49d6-4e56-900e-49f4bd4adc85",
    "description": "A function of seven variables; {a,b,c,d,x,y,z}.",
    "inputs": [
      {
        "name": "Function",
        "nickname": "F",
        "access": "item",
        "description": "Expression to solve"
      },
      {
        "name": "a",
        "nickname": "a",
        "access": "item",
        "description": "Variable #1"
      },
      {
        "name": "b",
        "nickname": "b",
        "access": "item",
        "description": "Variable #2"
      },
      {
        "name": "c",
        "nickname": "c",
        "access": "item",
        "description": "Variable #3"
      },
      {
        "name": "d",
        "nickname": "d",
        "access": "item",
        "description": "Variable #4"
      },
      {
        "name": "x",
        "nickname": "x",
        "access": "item",
        "description": "Variable #5"
      },
      {
        "name": "y",
        "nickname": "y",
        "access": "item",
        "description": "Variable #6"
      },
      {
        "name": "z",
        "nickname": "z",
        "access": "item",
        "description": "Variable #7"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "r",
        "access": "item",
        "description": "Expression result"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Boolean",
    "name": "Gate Or",
    "nickname": "Or",
    "guid": "eb3c8610-85b9-4593-a366-52550e8305b7",
    "description": "Perform boolean disjunction (OR gate).",
    "inputs": [
      {
        "name": "A",
        "nickname": "A",
        "access": "item",
        "description": "Left hand boolean"
      },
      {
        "name": "B",
        "nickname": "B",
        "access": "item",
        "description": "Right hand boolean"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "R",
        "access": "item",
        "description": "Resulting value"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Script",
    "name": "Variable Expression",
    "nickname": "Exp",
    "guid": "ef4ead41-6762-4adf-8a20-12b973bdf008",
    "description": "Expression component with a variable amount of input parameters.",
    "inputs": [
      {
        "name": "Variable a",
        "nickname": "a",
        "access": "item",
        "description": "Input expression variable"
      },
      {
        "name": "Variable b",
        "nickname": "b",
        "access": "item",
        "description": "Input expression variable"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "r",
        "access": "item",
        "description": "Expression result"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Domain",
    "name": "Deconstruct Domain²",
    "nickname": "DeDom2",
    "guid": "f0adfc96-b175-46a6-80c7-2b0ee17395c4",
    "description": "Deconstruct a two-dimensional domain into its component parts",
    "inputs": [
      {
        "name": "Domain",
        "nickname": "I",
        "access": "item",
        "description": "Base domain"
      }
    ],
    "outputs": [
      {
        "name": "U component",
        "nickname": "U",
        "access": "item",
        "description": "{u} component of domain"
      },
      {
        "name": "V component",
        "nickname": "V",
        "access": "item",
        "description": "{v} component of domain"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Time",
    "name": "Construct Smooth Time",
    "nickname": "SmTime",
    "guid": "f151b0b9-cef8-4809-96fc-9b14f1c3a7b9",
    "description": "Construct a time instance from smooth components",
    "inputs": [
      {
        "name": "Days",
        "nickname": "D",
        "access": "item",
        "description": "Number of days"
      },
      {
        "name": "Hours",
        "nickname": "H",
        "access": "item",
        "description": "Number of hours"
      },
      {
        "name": "Minutes",
        "nickname": "M",
        "access": "item",
        "description": "Number of minutes"
      },
      {
        "name": "Seconds",
        "nickname": "S",
        "access": "item",
        "description": "Number of seconds"
      }
    ],
    "outputs": [
      {
        "name": "Time",
        "nickname": "T",
        "access": "item",
        "description": "Time construct"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Domain",
    "name": "Includes",
    "nickname": "Inc",
    "guid": "f217f873-92f1-47ae-ad71-ca3c5a45c3f8",
    "description": "Test a numeric value to see if it is included in the domain",
    "inputs": [
      {
        "name": "Value",
        "nickname": "V",
        "access": "item",
        "description": "Value to test for inclusion"
      },
      {
        "name": "Domain",
        "nickname": "D",
        "access": "item",
        "description": "Domain to test with"
      }
    ],
    "outputs": [
      {
        "name": "Includes",
        "nickname": "I",
        "access": "item",
        "description": "True if the value is included in the domain"
      },
      {
        "name": "Deviation",
        "nickname": "D",
        "access": "item",
        "description": "Distance between the value and the nearest value inside the domain"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Script",
    "name": "F8",
    "nickname": "F(a,b,c,d,w,x,y,z)",
    "guid": "f2a97ac6-4f11-4c81-834d-50ecd782675c",
    "description": "A function of eight variables; {a,b,c,d,w,x,y,z}.",
    "inputs": [
      {
        "name": "Function",
        "nickname": "F",
        "access": "item",
        "description": "Expression to solve"
      },
      {
        "name": "a",
        "nickname": "a",
        "access": "item",
        "description": "Variable #1"
      },
      {
        "name": "b",
        "nickname": "b",
        "access": "item",
        "description": "Variable #2"
      },
      {
        "name": "c",
        "nickname": "c",
        "access": "item",
        "description": "Variable #3"
      },
      {
        "name": "d",
        "nickname": "d",
        "access": "item",
        "description": "Variable #4"
      },
      {
        "name": "w",
        "nickname": "w",
        "access": "item",
        "description": "Variable #5"
      },
      {
        "name": "x",
        "nickname": "x",
        "access": "item",
        "description": "Variable #6"
      },
      {
        "name": "y",
        "nickname": "y",
        "access": "item",
        "description": "Variable #7"
      },
      {
        "name": "z",
        "nickname": "z",
        "access": "item",
        "description": "Variable #8"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "r",
        "access": "item",
        "description": "Expression result"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Domain",
    "name": "Bounds",
    "nickname": "Bnd",
    "guid": "f44b92b0-3b5b-493a-86f4-fd7408c3daf3",
    "description": "Create a numeric domain which encompasses a list of numbers.",
    "inputs": [
      {
        "name": "Numbers",
        "nickname": "N",
        "access": "list",
        "description": "Numbers to include in Bounds"
      }
    ],
    "outputs": [
      {
        "name": "Domain",
        "nickname": "I",
        "access": "item",
        "description": "Numeric Domain between the lowest and highest numbers in {N}"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Script",
    "name": "C# Script",
    "nickname": "C#",
    "guid": "f5e3456b-dcfc-4faa-ac4e-7804cb75ee6d",
    "description": "A C#.NET scriptable component",
    "inputs": [
      {
        "name": "Variable x",
        "nickname": "x",
        "access": "item",
        "description": "Script Variable x"
      },
      {
        "name": "Variable y",
        "nickname": "y",
        "access": "item",
        "description": "Script Variable y"
      }
    ],
    "outputs": [
      {
        "name": "Output",
        "nickname": "out",
        "access": "item",
        "description": "Print, Reflect and Error streams"
      },
      {
        "name": "Result A",
        "nickname": "A",
        "access": "item",
        "description": "Output parameter A"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Matrix",
    "name": "Invert Matrix",
    "nickname": "MInvert",
    "guid": "f986e79a-1215-4822-a1e7-3311dbdeb851",
    "description": "Invert a matrix",
    "inputs": [
      {
        "name": "Matrix",
        "nickname": "M",
        "access": "item",
        "description": "Matrix to invert"
      },
      {
        "name": "Tolerance",
        "nickname": "t",
        "access": "item",
        "description": "Zero-tolerance for inversion"
      }
    ],
    "outputs": [
      {
        "name": "Matrix",
        "nickname": "M",
        "access": "item",
        "description": "Inverted matrix"
      },
      {
        "name": "Success",
        "nickname": "S",
        "access": "item",
        "description": "Boolean indicating inversion success"
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Domain",
    "name": "Remap Numbers",
    "nickname": "ReMap",
    "guid": "fa314286-867b-41fa-a7f6-3f474197bb81",
    "description": "Remap numbers into a new numeric domain",
    "inputs": [
      {
        "name": "Value",
        "nickname": "V",
        "access": "item",
        "description": "Value to remap"
      },
      {
        "name": "Source",
        "nickname": "S",
        "access": "item",
        "description": "Optional source domain."
      },
      {
        "name": "Target",
        "nickname": "T",
        "access": "item",
        "description": "Target domain"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "R",
        "access": "item",
        "description": "Remapped number."
      }
    ]
  },
  {
    "category": "Maths",
    "subcategory": "Script",
    "name": "DotNET VB Script (LEGACY)",
    "nickname": "VB",
    "guid": "fb6aba99-fead-4e42-b5d8-c6de5ff90ea6",
    "description": "A VB.NET scriptable component",
    "inputs": [
      {
        "name": "x",
        "nickname": "x",
        "access": "item",
        "description": "Script Variable x"
      },
      {
        "name": "y",
        "nickname": "y",
        "access": "item",
        "description": "Script Variable y"
      }
    ],
    "outputs": [
      {
        "name": "Output",
        "nickname": "out",
        "access": "item",
        "description": "Print, Reflect and Error streams"
      },
      {
        "name": "A",
        "nickname": "A",
        "access": "item",
        "description": "Output parameter A"
      }
    ]
  }
];
