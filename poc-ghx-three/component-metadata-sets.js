export const SETS_COMPONENTS = [
  {
    "category": "Sets",
    "subcategory": "Sequence",
    "name": "Cull Pattern",
    "nickname": "Cull",
    "guid": "008e9a6f-478a-4813-8c8a-546273bc3a6b",
    "description": "Cull (remove) elements in a list using a repeating bit mask.",
    "inputs": [
      {
        "name": "List",
        "nickname": "L",
        "access": "list",
        "description": "List to cull"
      },
      {
        "name": "Cull Pattern",
        "nickname": "P",
        "access": "list",
        "description": "Culling pattern"
      }
    ],
    "outputs": [
      {
        "name": "List",
        "nickname": "L",
        "access": "list",
        "description": "Culled list"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Sequence",
    "name": "Char Sequence",
    "nickname": "CharSeq",
    "guid": "01640871-69ea-40ac-9380-4660d6d28bd2",
    "description": "Create a sequence of textual characters.",
    "inputs": [
      {
        "name": "Count",
        "nickname": "C",
        "access": "item",
        "description": "Number of elements in the sequence."
      },
      {
        "name": "Char Pool",
        "nickname": "P",
        "access": "item",
        "description": "Pool of characters available to the sequence."
      },
      {
        "name": "Format",
        "nickname": "F",
        "access": "item",
        "description": "Optional formatting mask"
      }
    ],
    "outputs": [
      {
        "name": "Sequence",
        "nickname": "S",
        "access": "list",
        "description": "Sequence of character tags"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Text",
    "name": "Concatenate",
    "nickname": "Concat",
    "guid": "01cbd6e3-ccbe-4c24-baeb-46e10553e18b",
    "description": "Concatenate two fragments of text",
    "inputs": [
      {
        "name": "Start",
        "nickname": "A",
        "access": "item",
        "description": "Text to append to."
      },
      {
        "name": "End",
        "nickname": "B",
        "access": "item",
        "description": "Text to append."
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "R",
        "access": "item",
        "description": "Resulting text consisting of A+B"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "List",
    "name": "Pick'n'Choose",
    "nickname": "P'n'C",
    "guid": "03b801eb-87cd-476a-a591-257fe5d5bf0f",
    "description": "Pick and choose from a set of input data.",
    "inputs": [
      {
        "name": "Pattern",
        "nickname": "P",
        "access": "list",
        "description": "Pick pattern of input indices"
      },
      {
        "name": "Stream 0",
        "nickname": "0",
        "access": "list",
        "description": "Input stream 0"
      },
      {
        "name": "Stream 1",
        "nickname": "1",
        "access": "list",
        "description": "Input stream 1"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "R",
        "access": "list",
        "description": "Picked result"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Text",
    "name": "Text Split",
    "nickname": "Split",
    "guid": "04887d01-504c-480e-b2a2-01ea19cc5922",
    "description": "Split some text into fragments using separators",
    "inputs": [
      {
        "name": "Text",
        "nickname": "T",
        "access": "item",
        "description": "Text to split."
      },
      {
        "name": "Separators",
        "nickname": "C",
        "access": "item",
        "description": "Separator characters."
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "R",
        "access": "list",
        "description": "Resulting text fragments"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Tree",
    "name": "Simplify Tree",
    "nickname": "Simplify",
    "guid": "06b3086c-1e9d-41c2-bcfc-bb843156196e",
    "description": "Simplify a tree by removing the overlap shared amongst all branches.",
    "inputs": [
      {
        "name": "Tree",
        "nickname": "T",
        "access": "tree",
        "description": "Tree to simplify."
      }
    ],
    "outputs": [
      {
        "name": "Tree",
        "nickname": "T",
        "access": "tree",
        "description": "Simplified tree."
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Tree",
    "name": "Clean Tree",
    "nickname": "Clean",
    "guid": "071c3940-a12d-4b77-bb23-42b5d3314a0d",
    "description": "Removed all null and invalid items from a data tree.",
    "inputs": [
      {
        "name": "Remove Nulls",
        "nickname": "N",
        "access": "item",
        "description": "Remove null items from the tree."
      },
      {
        "name": "Remove Invalid",
        "nickname": "X",
        "access": "item",
        "description": "Remove invalid items from the tree."
      },
      {
        "name": "Remove Empty",
        "nickname": "E",
        "access": "item",
        "description": "Remove empty branches from the tree."
      },
      {
        "name": "Tree",
        "nickname": "T",
        "access": "tree",
        "description": "Data tree to clean"
      }
    ],
    "outputs": [
      {
        "name": "Tree",
        "nickname": "T",
        "access": "tree",
        "description": "Spotless data tree"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Text",
    "name": "Text Fragment",
    "nickname": "Fragment",
    "guid": "07e0811f-034a-4504-bca0-2d03b2c46217",
    "description": "Extract a fragment (subset) of some text",
    "inputs": [
      {
        "name": "Text",
        "nickname": "T",
        "access": "item",
        "description": "Text to operate on."
      },
      {
        "name": "Start",
        "nickname": "i",
        "access": "item",
        "description": "Zero based index of first character to copy."
      },
      {
        "name": "Count",
        "nickname": "N",
        "access": "item",
        "description": "Optional number of characters to copy. If blank, the entire remainder will be copied."
      }
    ],
    "outputs": [
      {
        "name": "Fragment",
        "nickname": "F",
        "access": "item",
        "description": "The resulting text fragment"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Tree",
    "name": "Merge Multiple",
    "nickname": "Merge",
    "guid": "0b6c5dac-6c93-4158-b8d1-ca3187d45f25",
    "description": "Merge multiple input streams into one",
    "inputs": [
      {
        "name": "Stream 0",
        "nickname": "0",
        "access": "tree",
        "description": "Input stream #1"
      },
      {
        "name": "Stream 1",
        "nickname": "1",
        "access": "tree",
        "description": "Input stream #2"
      }
    ],
    "outputs": [
      {
        "name": "Stream",
        "nickname": "S",
        "access": "tree",
        "description": "Merged stream"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Tree",
    "name": "Graft Tree",
    "nickname": "Graft",
    "guid": "10a8674b-f4bb-4fdf-a56e-94dc606ecf33",
    "description": "Graft a tree by adding an extra branch for every data item.",
    "inputs": [
      {
        "name": "Data",
        "nickname": "D",
        "access": "tree",
        "description": "Data to graft"
      },
      {
        "name": "Strip",
        "nickname": "S",
        "access": "item",
        "description": "Do not create branches for null items"
      }
    ],
    "outputs": [
      {
        "name": "Tree",
        "nickname": "T",
        "access": "item",
        "description": "Graft result"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Tree",
    "name": "Trim Tree",
    "nickname": "Trim",
    "guid": "1177d6ee-3993-4226-9558-52b7fd63e1e3",
    "description": "Reduce the complexity of a tree by merging the outermost branches.",
    "inputs": [
      {
        "name": "Tree",
        "nickname": "T",
        "access": "tree",
        "description": "Data tree to flatten"
      },
      {
        "name": "Depth",
        "nickname": "D",
        "access": "item",
        "description": "Number of outermost branches to merge"
      }
    ],
    "outputs": [
      {
        "name": "Tree",
        "nickname": "T",
        "access": "tree",
        "description": "Trimmed data tree"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Text",
    "name": "Text Join",
    "nickname": "Join",
    "guid": "1274d51a-81e6-4ccf-ad1f-0edf4c769cac",
    "description": "Join a collection of text fragments into one",
    "inputs": [
      {
        "name": "Text",
        "nickname": "T",
        "access": "list",
        "description": "Text fragments to join."
      },
      {
        "name": "Join",
        "nickname": "J",
        "access": "item",
        "description": "Fragment separator."
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "R",
        "access": "item",
        "description": "Resulting text"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Tree",
    "name": "Simplify Tree",
    "nickname": "Simplify",
    "guid": "1303da7b-e339-4e65-a051-82c4dce8224d",
    "description": "Simplify a data tree by removing the overlap shared amongst all branches.",
    "inputs": [
      {
        "name": "Tree",
        "nickname": "T",
        "access": "tree",
        "description": "Data tree to simplify."
      },
      {
        "name": "Front",
        "nickname": "F",
        "access": "item",
        "description": "Limit path collapse to indices at the start of the path only."
      }
    ],
    "outputs": [
      {
        "name": "Tree",
        "nickname": "T",
        "access": "tree",
        "description": "Simplified data tree."
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "List",
    "name": "Weave",
    "nickname": "Weave",
    "guid": "160c1df2-e2e8-48e5-b538-f2d6981007e3",
    "description": "Weave a set of input streams using a custom pattern.",
    "inputs": [
      {
        "name": "Pattern",
        "nickname": "P",
        "access": "list",
        "description": "Weave pattern of input indices"
      },
      {
        "name": "Stream 0",
        "nickname": "0",
        "access": "list",
        "description": "Input stream 0"
      },
      {
        "name": "Stream 1",
        "nickname": "1",
        "access": "list",
        "description": "Input stream 1"
      }
    ],
    "outputs": [
      {
        "name": "Weave",
        "nickname": "W",
        "access": "list",
        "description": "Weave result"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "List",
    "name": "List Length",
    "nickname": "Lng",
    "guid": "1817fd29-20ae-4503-b542-f0fb651e67d7",
    "description": "Measure the length of a list.",
    "inputs": [
      {
        "name": "List",
        "nickname": "L",
        "access": "list",
        "description": "Base list"
      }
    ],
    "outputs": [
      {
        "name": "Length",
        "nickname": "L",
        "access": "item",
        "description": "Number of items in L"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Sets",
    "name": "Delete Consecutive",
    "nickname": "DCon",
    "guid": "190d042c-2270-4bc1-81c0-4f90c170c9c9",
    "description": "Delete consecutive similar members in a set.",
    "inputs": [
      {
        "name": "Set",
        "nickname": "S",
        "access": "list",
        "description": "Set to operate on."
      },
      {
        "name": "Wrap",
        "nickname": "W",
        "access": "item",
        "description": "If true, the last and first member are considered to be adjacent."
      }
    ],
    "outputs": [
      {
        "name": "Set",
        "nickname": "S",
        "access": "list",
        "description": "Set with consecutive identical members removed."
      },
      {
        "name": "Count",
        "nickname": "N",
        "access": "item",
        "description": "Number of members removed."
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Streams",
    "name": "Separate List [OBSOLETE]",
    "nickname": "Separate",
    "guid": "1d55b516-ceec-4bf1-8864-08c895ab2a70",
    "description": "Separate the items in a list with a custom filter.",
    "inputs": [
      {
        "name": "Data",
        "nickname": "D",
        "access": "item",
        "description": "Base list"
      },
      {
        "name": "Flag",
        "nickname": "F",
        "access": "item",
        "description": "Split redirection flag"
      }
    ],
    "outputs": [
      {
        "name": "True",
        "nickname": "T",
        "access": "item",
        "description": "Items for {F = True}"
      },
      {
        "name": "False",
        "nickname": "F",
        "access": "item",
        "description": "Items for {F = False}"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Tree",
    "name": "Path Compare",
    "nickname": "Compare",
    "guid": "1d8b0e2c-e772-4fa9-b7f7-b158251b34b8",
    "description": "Compare a path to a mask pattern",
    "inputs": [
      {
        "name": "Path",
        "nickname": "P",
        "access": "item",
        "description": "Path to compare"
      },
      {
        "name": "Mask",
        "nickname": "M",
        "access": "item",
        "description": "Comparison mask"
      }
    ],
    "outputs": [
      {
        "name": "Comparison",
        "nickname": "C",
        "access": "item",
        "description": "Comparison (True = Match, False = Mismatch)"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Sets",
    "name": "Key/Value Search",
    "nickname": "KeySearch",
    "guid": "1edcc3cf-cf84-41d4-8204-561162cfe510",
    "description": "Extract an item from a collection using a key-value match",
    "inputs": [
      {
        "name": "Keys",
        "nickname": "K",
        "access": "list",
        "description": "A list of key values."
      },
      {
        "name": "Values",
        "nickname": "V",
        "access": "list",
        "description": "A list of value data, one for each key."
      },
      {
        "name": "Search",
        "nickname": "S",
        "access": "item",
        "description": "A key value to search for"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "R",
        "access": "item",
        "description": "Resulting item in the value list that matches the Search key"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Text",
    "name": "Sort Text",
    "nickname": "TSort",
    "guid": "1ff80a00-1b1d-4fb3-926a-0c246261fc55",
    "description": "Sort a collection of text fragments",
    "inputs": [
      {
        "name": "Keys",
        "nickname": "K",
        "access": "list",
        "description": "Text fragments to sort (sorting key)"
      },
      {
        "name": "Values",
        "nickname": "V",
        "access": "list",
        "description": "Optional values to sort synchronously"
      }
    ],
    "outputs": [
      {
        "name": "Keys",
        "nickname": "K",
        "access": "list",
        "description": "Sorted text fragments"
      },
      {
        "name": "Values",
        "nickname": "V",
        "access": "list",
        "description": "Sorted values"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Text",
    "name": "Concatenate",
    "nickname": "Concat",
    "guid": "2013e425-8713-42e2-a661-b57e78840337",
    "description": "Concatenate some fragments of text",
    "inputs": [
      {
        "name": "Fragment A",
        "nickname": "A",
        "access": "item",
        "description": "First text fragment"
      },
      {
        "name": "Fragment B",
        "nickname": "B",
        "access": "item",
        "description": "Second text fragment"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "R",
        "access": "item",
        "description": "Resulting text consisting of all the fragments"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Tree",
    "name": "Merge 10",
    "nickname": "M10",
    "guid": "22f66ff6-d281-453c-bd8c-36ed24026783",
    "description": "Merge ten streams into one.",
    "inputs": [
      {
        "name": "Stream A",
        "nickname": "A",
        "access": "tree",
        "description": "Input stream #1"
      },
      {
        "name": "Stream B",
        "nickname": "B",
        "access": "tree",
        "description": "Input stream #2"
      },
      {
        "name": "Stream C",
        "nickname": "C",
        "access": "tree",
        "description": "Input stream #3"
      },
      {
        "name": "Stream D",
        "nickname": "D",
        "access": "tree",
        "description": "Input stream #4"
      },
      {
        "name": "Stream E",
        "nickname": "E",
        "access": "tree",
        "description": "Input stream #5"
      },
      {
        "name": "Stream F",
        "nickname": "F",
        "access": "tree",
        "description": "Input stream #6"
      },
      {
        "name": "Stream G",
        "nickname": "G",
        "access": "tree",
        "description": "Input stream #7"
      },
      {
        "name": "Stream H",
        "nickname": "H",
        "access": "tree",
        "description": "Input stream #8"
      },
      {
        "name": "Stream I",
        "nickname": "I",
        "access": "tree",
        "description": "Input stream #9"
      },
      {
        "name": "Stream J",
        "nickname": "J",
        "access": "tree",
        "description": "Input stream #10"
      }
    ],
    "outputs": [
      {
        "name": "Stream",
        "nickname": "S",
        "access": "item",
        "description": "Merged stream"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Tree",
    "name": "Relative Items",
    "nickname": "RelItem2",
    "guid": "2653b135-4df1-4a6b-820c-55e2ad3bc1e0",
    "description": "Retrieve a relative item combo from two data trees",
    "inputs": [
      {
        "name": "Tree A",
        "nickname": "A",
        "access": "tree",
        "description": "First Data Tree"
      },
      {
        "name": "Tree B",
        "nickname": "B",
        "access": "tree",
        "description": "Second Data Tree"
      },
      {
        "name": "Offset",
        "nickname": "O",
        "access": "item",
        "description": "Relative offset for item combo"
      },
      {
        "name": "Wrap Paths",
        "nickname": "Wp",
        "access": "item",
        "description": "Wrap paths when the shift is out of bounds"
      },
      {
        "name": "Wrap Items",
        "nickname": "Wi",
        "access": "item",
        "description": "Wrap items when the shift is out of bounds"
      }
    ],
    "outputs": [
      {
        "name": "Item A",
        "nickname": "A",
        "access": "tree",
        "description": "Item in tree A"
      },
      {
        "name": "Item B",
        "nickname": "B",
        "access": "tree",
        "description": "Relative item in tree B"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Text",
    "name": "Text On Surface",
    "nickname": "TextSrf",
    "guid": "28504f1f-a8d9-40c8-b8aa-529413456258",
    "description": "Create a collection of textual symbols aligned on a surface.",
    "inputs": [
      {
        "name": "Text",
        "nickname": "T",
        "access": "item",
        "description": "Text to create."
      },
      {
        "name": "Font",
        "nickname": "F",
        "access": "item",
        "description": "Font name, with optional 'Bold' or 'Italic' tags."
      },
      {
        "name": "Height",
        "nickname": "H",
        "access": "item",
        "description": "Height of text shapes."
      },
      {
        "name": "Depth",
        "nickname": "D",
        "access": "item",
        "description": "Depth of text shapes."
      },
      {
        "name": "Base Line",
        "nickname": "B",
        "access": "item",
        "description": "Base line for text."
      },
      {
        "name": "Base Surface",
        "nickname": "S",
        "access": "item",
        "description": "Optional base surface for text orientation. Surfaces, meshes and SubDs are all allowed."
      }
    ],
    "outputs": [
      {
        "name": "Symbols",
        "nickname": "S",
        "access": "list",
        "description": "Symbols making up the text shapes."
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "List",
    "name": "List Item",
    "nickname": "Item",
    "guid": "285ddd8a-5398-4a3e-b3c2-361025711a51",
    "description": "Retrieve a specific item from a list.",
    "inputs": [
      {
        "name": "List",
        "nickname": "L",
        "access": "list",
        "description": "Base list"
      },
      {
        "name": "Index",
        "nickname": "i",
        "access": "item",
        "description": "Item index"
      },
      {
        "name": "Wrap",
        "nickname": "W",
        "access": "item",
        "description": "Wrap index to list bounds"
      }
    ],
    "outputs": [
      {
        "name": "Element",
        "nickname": "E",
        "access": "item",
        "description": "Item at {i'}"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Sequence",
    "name": "Random",
    "nickname": "Random",
    "guid": "2ab17f9a-d852-4405-80e1-938c5e57e78d",
    "description": "Generate a list of pseudo random numbers.",
    "inputs": [
      {
        "name": "Range",
        "nickname": "R",
        "access": "item",
        "description": "Domain of random numeric range"
      },
      {
        "name": "Number",
        "nickname": "N",
        "access": "item",
        "description": "Number of random values"
      },
      {
        "name": "Seed",
        "nickname": "S",
        "access": "item",
        "description": "Seed of random engine"
      }
    ],
    "outputs": [
      {
        "name": "Random",
        "nickname": "R",
        "access": "list",
        "description": "Random numbers"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "List",
    "name": "Sort List",
    "nickname": "Sort",
    "guid": "2b2628ea-3f43-4ce9-8435-9a045d54b5c6",
    "description": "Sort a list of numeric keys.",
    "inputs": [
      {
        "name": "Keys",
        "nickname": "K",
        "access": "list",
        "description": "List of sortable keys"
      },
      {
        "name": "Values A",
        "nickname": "A",
        "access": "list",
        "description": "Optional list to sort synchronously"
      },
      {
        "name": "Values B",
        "nickname": "B",
        "access": "list",
        "description": "Optional list to sort synchronously"
      },
      {
        "name": "Values C",
        "nickname": "C",
        "access": "list",
        "description": "Optional list to sort synchronously"
      }
    ],
    "outputs": [
      {
        "name": "List",
        "nickname": "L",
        "access": "list",
        "description": "Sorted keys"
      },
      {
        "name": "Values A",
        "nickname": "A",
        "access": "list",
        "description": "Synchronous values in A"
      },
      {
        "name": "Values B",
        "nickname": "B",
        "access": "list",
        "description": "Synchronous values in B"
      },
      {
        "name": "Values C",
        "nickname": "C",
        "access": "list",
        "description": "Synchronous values in C"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Sets",
    "name": "Create Set",
    "nickname": "CSet",
    "guid": "2cb4bf85-a282-464c-b42c-8e735d2a0a74",
    "description": "Creates the valid set from a list of items (a valid set only contains distinct elements).",
    "inputs": [
      {
        "name": "List",
        "nickname": "L",
        "access": "list",
        "description": "List of data."
      }
    ],
    "outputs": [
      {
        "name": "Set",
        "nickname": "S",
        "access": "list",
        "description": "A set of all the distincts values in L."
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Tree",
    "name": "Shift Paths",
    "nickname": "PShift",
    "guid": "2d61f4e0-47c5-41d6-a41d-6afa96ee63af",
    "description": "Shift the indices in all data tree paths",
    "inputs": [
      {
        "name": "Data",
        "nickname": "D",
        "access": "tree",
        "description": "Data to modify"
      },
      {
        "name": "Offset",
        "nickname": "O",
        "access": "item",
        "description": "Offset to apply to each branch"
      }
    ],
    "outputs": [
      {
        "name": "Data",
        "nickname": "D",
        "access": "tree",
        "description": "Shifted data"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "List",
    "name": "Sift Pattern",
    "nickname": "Sift",
    "guid": "3249222f-f536-467a-89f4-f0353fba455a",
    "description": "Sift elements in a list using a repeating index pattern.",
    "inputs": [
      {
        "name": "List",
        "nickname": "L",
        "access": "list",
        "description": "List to sift"
      },
      {
        "name": "Sift Pattern",
        "nickname": "P",
        "access": "list",
        "description": "Sifting pattern"
      }
    ],
    "outputs": [
      {
        "name": "Output 0",
        "nickname": "0",
        "access": "list",
        "description": "Output for sift index 0"
      },
      {
        "name": "Output 1",
        "nickname": "1",
        "access": "list",
        "description": "Output for sift index 1"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "List",
    "name": "Cross Reference",
    "nickname": "CrossRef",
    "guid": "36947590-f0cb-4807-a8f9-9c90c9b20621",
    "description": "Cross Reference data from multiple lists",
    "inputs": [
      {
        "name": "List (A)",
        "nickname": "A",
        "access": "list",
        "description": "List (A) to operate on"
      },
      {
        "name": "List (B)",
        "nickname": "B",
        "access": "list",
        "description": "List (B) to operate on"
      }
    ],
    "outputs": [
      {
        "name": "List (A)",
        "nickname": "A",
        "access": "list",
        "description": "Adjusted list (A)"
      },
      {
        "name": "List (B)",
        "nickname": "B",
        "access": "list",
        "description": "Adjusted list (B)"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Text",
    "name": "Match Text",
    "nickname": "TMatch",
    "guid": "3756c55f-95c3-442c-a027-6b3ab0455a94",
    "description": "Match a text against a pattern",
    "inputs": [
      {
        "name": "Text",
        "nickname": "T",
        "access": "item",
        "description": "Text to match"
      },
      {
        "name": "Pattern",
        "nickname": "P",
        "access": "item",
        "description": "Optional wildcard pattern for matching"
      },
      {
        "name": "RegEx",
        "nickname": "R",
        "access": "item",
        "description": "Optional RegEx pattern for matching"
      },
      {
        "name": "Case",
        "nickname": "C",
        "access": "item",
        "description": "Compare using case-sensitive matching"
      }
    ],
    "outputs": [
      {
        "name": "Match",
        "nickname": "M",
        "access": "item",
        "description": "True if the text adheres to all supplied patterns"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Tree",
    "name": "Tree Branch",
    "nickname": "Branch",
    "guid": "3a710c1e-1809-4e19-8c15-82adce31cd62",
    "description": "Retrieve a specific branch from a data tree.",
    "inputs": [
      {
        "name": "Tree",
        "nickname": "T",
        "access": "tree",
        "description": "Data Tree"
      },
      {
        "name": "Path",
        "nickname": "P",
        "access": "item",
        "description": "Data tree branch path"
      }
    ],
    "outputs": [
      {
        "name": "Branch",
        "nickname": "B",
        "access": "tree",
        "description": "Branch at {P}"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Tree",
    "name": "Merge",
    "nickname": "Merge",
    "guid": "3cadddef-1e2b-4c09-9390-0e8f78f7609f",
    "description": "Merge a bunch of data streams",
    "inputs": [
      {
        "name": "Data 1",
        "nickname": "D1",
        "access": "tree",
        "description": "Data stream 1"
      },
      {
        "name": "Data 2",
        "nickname": "D2",
        "access": "tree",
        "description": "Data stream 2"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "R",
        "access": "tree",
        "description": "Result of merge"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Tree",
    "name": "Stream Filter",
    "nickname": "Filter",
    "guid": "3e5582a1-901a-4f7c-b58d-f5d7e3166124",
    "description": "Filters a collection of input streams",
    "inputs": [
      {
        "name": "Gate",
        "nickname": "G",
        "access": "item",
        "description": "Index of Gate stream"
      },
      {
        "name": "Stream 0",
        "nickname": "0",
        "access": "tree",
        "description": "Input stream at index 0"
      },
      {
        "name": "Stream 1",
        "nickname": "1",
        "access": "tree",
        "description": "Input stream at index 1"
      }
    ],
    "outputs": [
      {
        "name": "Stream",
        "nickname": "S",
        "access": "tree",
        "description": "Filtered stream"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Sets",
    "name": "Member Index",
    "nickname": "MIndex",
    "guid": "3ff27857-b988-417a-b495-b24c733dbd00",
    "description": "Find the occurences of a specific member in a set.",
    "inputs": [
      {
        "name": "Set",
        "nickname": "S",
        "access": "list",
        "description": "Set to operate on."
      },
      {
        "name": "Member",
        "nickname": "M",
        "access": "item",
        "description": "Member to search for."
      }
    ],
    "outputs": [
      {
        "name": "Index",
        "nickname": "I",
        "access": "list",
        "description": "Indices of member."
      },
      {
        "name": "Count",
        "nickname": "N",
        "access": "item",
        "description": "Number of occurences of the member."
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Tree",
    "name": "Flip Matrix",
    "nickname": "Flip",
    "guid": "41aa4112-9c9b-42f4-847e-503b9d90e4c7",
    "description": "Flip a matrix-like data tree by swapping rows and columns.",
    "inputs": [
      {
        "name": "Data",
        "nickname": "D",
        "access": "tree",
        "description": "Data matrix to flip"
      }
    ],
    "outputs": [
      {
        "name": "Data",
        "nickname": "D",
        "access": "tree",
        "description": "Flipped data matrix"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "List",
    "name": "Pick'n'Choose",
    "nickname": "P'n'C",
    "guid": "4356ef8f-0ca1-4632-9c39-9e6dcd2b9496",
    "description": "Pick and choose from a set of input lists.",
    "inputs": [
      {
        "name": "Pattern",
        "nickname": "P",
        "access": "list",
        "description": "Pick pattern of input indices"
      },
      {
        "name": "Stream 0",
        "nickname": "0",
        "access": "list",
        "description": "Input stream 0"
      },
      {
        "name": "Stream 1",
        "nickname": "1",
        "access": "list",
        "description": "Input stream 1"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "R",
        "access": "list",
        "description": "Picked result"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Sequence",
    "name": "Random Reduce",
    "nickname": "Reduce",
    "guid": "455925fd-23ff-4e57-a0e7-913a4165e659",
    "description": "Randomly remove N items from a list",
    "inputs": [
      {
        "name": "List",
        "nickname": "L",
        "access": "list",
        "description": "List to reduce"
      },
      {
        "name": "Reduction",
        "nickname": "R",
        "access": "item",
        "description": "Number of items to remove"
      },
      {
        "name": "Seed",
        "nickname": "S",
        "access": "item",
        "description": "Random Generator Seed value"
      }
    ],
    "outputs": [
      {
        "name": "List",
        "nickname": "L",
        "access": "list",
        "description": "Reduced list"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Tree",
    "name": "Match Tree",
    "nickname": "Match",
    "guid": "46372d0d-82dc-4acb-adc3-25d1fde04c4e",
    "description": "Match one data tree with another.",
    "inputs": [
      {
        "name": "Tree",
        "nickname": "T",
        "access": "tree",
        "description": "Data tree to modify"
      },
      {
        "name": "Guide",
        "nickname": "G",
        "access": "tree",
        "description": "Data tree to match"
      }
    ],
    "outputs": [
      {
        "name": "Tree",
        "nickname": "T",
        "access": "tree",
        "description": "Matched data tree containing the data of T but the layout of G"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Tree",
    "name": "Merge 03",
    "nickname": "M3",
    "guid": "481f0339-1299-43ba-b15c-c07891a8f822",
    "description": "Merge three streams into one.",
    "inputs": [
      {
        "name": "Stream A",
        "nickname": "A",
        "access": "tree",
        "description": "Input stream #1"
      },
      {
        "name": "Stream B",
        "nickname": "B",
        "access": "tree",
        "description": "Input stream #2"
      },
      {
        "name": "Stream C",
        "nickname": "C",
        "access": "tree",
        "description": "Input stream #3"
      }
    ],
    "outputs": [
      {
        "name": "Stream",
        "nickname": "S",
        "access": "item",
        "description": "Merged stream"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Sets",
    "name": "SubSet",
    "nickname": "SubSet",
    "guid": "4cfc0bb0-0745-4772-a520-39f9bf3d99bc",
    "description": "Test two sets for inclusion.",
    "inputs": [
      {
        "name": "Set A",
        "nickname": "A",
        "access": "list",
        "description": "Super set."
      },
      {
        "name": "Set B",
        "nickname": "B",
        "access": "list",
        "description": "Sub set."
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "R",
        "access": "item",
        "description": "True if all items in B are present in A."
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Text",
    "name": "Replace Text",
    "nickname": "Rep",
    "guid": "4df8df00-3635-45bd-95e6-f9206296c110",
    "description": "Replace all occurences of a specific text fragment with another",
    "inputs": [
      {
        "name": "Text",
        "nickname": "T",
        "access": "item",
        "description": "Text to operate on."
      },
      {
        "name": "Find",
        "nickname": "F",
        "access": "item",
        "description": "Fragment to replace."
      },
      {
        "name": "Replace",
        "nickname": "R",
        "access": "item",
        "description": "Optional fragment to replace with. If blank, all occurences of F will be removed."
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "R",
        "access": "item",
        "description": "Result of text replacement"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "List",
    "name": "Shift List",
    "nickname": "Shift",
    "guid": "4fdfe351-6c07-47ce-9fb9-be027fb62186",
    "description": "Offset all items in a list.",
    "inputs": [
      {
        "name": "List",
        "nickname": "L",
        "access": "list",
        "description": "List to shift"
      },
      {
        "name": "Shift",
        "nickname": "S",
        "access": "item",
        "description": "Shift offset"
      },
      {
        "name": "Wrap",
        "nickname": "W",
        "access": "item",
        "description": "Wrap values"
      }
    ],
    "outputs": [
      {
        "name": "List",
        "nickname": "L",
        "access": "list",
        "description": "Shifted list"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Sequence",
    "name": "Cull Index",
    "nickname": "Cull i",
    "guid": "501aecbb-c191-4d13-83d6-7ee32445ac50",
    "description": "Cull (remove) indexed elements from a list.",
    "inputs": [
      {
        "name": "List",
        "nickname": "L",
        "access": "list",
        "description": "List to cull"
      },
      {
        "name": "Indices",
        "nickname": "I",
        "access": "list",
        "description": "Culling indices"
      },
      {
        "name": "Wrap",
        "nickname": "W",
        "access": "item",
        "description": "Wrap indices to list range"
      }
    ],
    "outputs": [
      {
        "name": "List",
        "nickname": "L",
        "access": "list",
        "description": "Culled list"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "List",
    "name": "Weave",
    "nickname": "Weave",
    "guid": "50faccbd-9c92-4175-a5fa-d65e36013db6",
    "description": "Weave a set of input data using a custom pattern.",
    "inputs": [
      {
        "name": "Pattern",
        "nickname": "P",
        "access": "list",
        "description": "Weave pattern of input indices"
      },
      {
        "name": "Stream 0",
        "nickname": "0",
        "access": "list",
        "description": "Input stream  0"
      },
      {
        "name": "Stream 1",
        "nickname": "1",
        "access": "list",
        "description": "Input stream  1"
      }
    ],
    "outputs": [
      {
        "name": "Weave",
        "nickname": "W",
        "access": "list",
        "description": "Weave result"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "List",
    "name": "List Item",
    "nickname": "Item",
    "guid": "59daf374-bc21-4a5e-8282-5504fb7ae9ae",
    "description": "Retrieve a specific item from a list.",
    "inputs": [
      {
        "name": "List",
        "nickname": "L",
        "access": "list",
        "description": "Base list"
      },
      {
        "name": "Index",
        "nickname": "i",
        "access": "item",
        "description": "Item index"
      },
      {
        "name": "Wrap",
        "nickname": "W",
        "access": "item",
        "description": "Wrap index to list bounds"
      }
    ],
    "outputs": [
      {
        "name": "Item",
        "nickname": "i",
        "access": "item",
        "description": "Item at {i'}"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "List",
    "name": "Shortest List",
    "nickname": "Short",
    "guid": "5a13ec19-e4e9-43da-bf65-f93025fa87ca",
    "description": "Shrink a collection of lists to the shortest length amongst them",
    "inputs": [
      {
        "name": "List (A)",
        "nickname": "A",
        "access": "list",
        "description": "List (A) to operate on"
      },
      {
        "name": "List (B)",
        "nickname": "B",
        "access": "list",
        "description": "List (B) to operate on"
      }
    ],
    "outputs": [
      {
        "name": "List (A)",
        "nickname": "A",
        "access": "list",
        "description": "Adjusted list (A)"
      },
      {
        "name": "List (B)",
        "nickname": "B",
        "access": "list",
        "description": "Adjusted list (B)"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "List",
    "name": "Partition List",
    "nickname": "Partition",
    "guid": "5a93246d-2595-4c28-bc2d-90657634f92a",
    "description": "Partition a list into sub-lists",
    "inputs": [
      {
        "name": "List",
        "nickname": "L",
        "access": "list",
        "description": "List to partition"
      },
      {
        "name": "Size",
        "nickname": "S",
        "access": "list",
        "description": "Size of partitions"
      }
    ],
    "outputs": [
      {
        "name": "Chunks",
        "nickname": "C",
        "access": "tree",
        "description": "List chunks"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Sequence",
    "name": "Stack Data",
    "nickname": "Stack",
    "guid": "5fa4e736-0d82-4af0-97fb-30a79f4cbf41",
    "description": "Duplicate individual items in a list of data",
    "inputs": [
      {
        "name": "Data",
        "nickname": "D",
        "access": "list",
        "description": "Data to stack"
      },
      {
        "name": "Stack",
        "nickname": "S",
        "access": "list",
        "description": "Stacking pattern"
      }
    ],
    "outputs": [
      {
        "name": "Data",
        "nickname": "D",
        "access": "list",
        "description": "Stacked data"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Sequence",
    "name": "Cull Index",
    "nickname": "Cull i",
    "guid": "6568e019-f59c-4984-84d6-96bd5bfbe9e7",
    "description": "Cull (remove) indexed elements from a list.",
    "inputs": [
      {
        "name": "List",
        "nickname": "L",
        "access": "list",
        "description": "List to cull"
      },
      {
        "name": "Indices",
        "nickname": "I",
        "access": "list",
        "description": "Culling indices"
      }
    ],
    "outputs": [
      {
        "name": "List",
        "nickname": "L",
        "access": "list",
        "description": "Culled list"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "List",
    "name": "Null Item",
    "nickname": "Null",
    "guid": "66fbaae1-0fcf-4dbf-bcba-4395d8f6a3e6",
    "description": "Test a data item for null or invalidity",
    "inputs": [
      {
        "name": "Items",
        "nickname": "I",
        "access": "tree",
        "description": "Items to test"
      }
    ],
    "outputs": [
      {
        "name": "Null Flags",
        "nickname": "N",
        "access": "item",
        "description": "True if item is Null"
      },
      {
        "name": "Invalid Flags",
        "nickname": "X",
        "access": "item",
        "description": "True if item is Invalid"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "List",
    "name": "List item [OBSOLETE]",
    "nickname": "Item",
    "guid": "6e2ba21a-2252-42f4-8d3f-f5e0f49cc4ef",
    "description": "This component is obsolete. It has been replaced with a new ITEM component",
    "inputs": [
      {
        "name": "List",
        "nickname": "L",
        "access": "list",
        "description": "The input list"
      },
      {
        "name": "Index",
        "nickname": "i",
        "access": "item",
        "description": "The index to retrieve"
      }
    ],
    "outputs": [
      {
        "name": "Element",
        "nickname": "E",
        "access": "item",
        "description": "The element at L(i)"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "List",
    "name": "Reverse List",
    "nickname": "Rev",
    "guid": "6ec97ea8-c559-47a2-8d0f-ce80c794d1f4",
    "description": "Reverse the order of a list.",
    "inputs": [
      {
        "name": "List",
        "nickname": "L",
        "access": "list",
        "description": "Base list"
      }
    ],
    "outputs": [
      {
        "name": "List",
        "nickname": "L",
        "access": "list",
        "description": "Reversed list"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "List",
    "name": "Sort List",
    "nickname": "Sort",
    "guid": "6f93d366-919f-4dda-a35e-ba03dd62799b",
    "description": "Sort a list of numeric keys.",
    "inputs": [
      {
        "name": "Keys",
        "nickname": "K",
        "access": "list",
        "description": "List of sortable keys"
      },
      {
        "name": "Values A",
        "nickname": "A",
        "access": "list",
        "description": "Optional list of values to sort synchronously"
      }
    ],
    "outputs": [
      {
        "name": "Keys",
        "nickname": "K",
        "access": "list",
        "description": "Sorted keys"
      },
      {
        "name": "Values A",
        "nickname": "A",
        "access": "list",
        "description": "Synchronous values in A"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Tree",
    "name": "Clean Tree",
    "nickname": "Clean",
    "guid": "70ce4230-da08-4fce-b29d-63dc42a88585",
    "description": "Remove all null and invalid entries from a Data Tree.",
    "inputs": [
      {
        "name": "Tree",
        "nickname": "T",
        "access": "tree",
        "description": "Data Tree to clean"
      },
      {
        "name": "Invalid",
        "nickname": "X",
        "access": "item",
        "description": "Remove invalid entries in addition to null entries."
      }
    ],
    "outputs": [
      {
        "name": "Data",
        "nickname": "D",
        "access": "item",
        "description": "Spotless data"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Tree",
    "name": "Stream Gate",
    "nickname": "Gate",
    "guid": "71fcc052-6add-4d70-8d97-cfb37ea9d169",
    "description": "Redirects a stream into specific outputs.",
    "inputs": [
      {
        "name": "Stream",
        "nickname": "S",
        "access": "tree",
        "description": "Input stream"
      },
      {
        "name": "Gate",
        "nickname": "G",
        "access": "item",
        "description": "Gate index of output stream"
      }
    ],
    "outputs": [
      {
        "name": "Target 0",
        "nickname": "0",
        "access": "tree",
        "description": "Output for Gate index 0"
      },
      {
        "name": "Target 1",
        "nickname": "1",
        "access": "tree",
        "description": "Output for Gate index 1"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Tree",
    "name": "Explode Tree",
    "nickname": "BANG!",
    "guid": "74cad441-2264-45fe-a57d-85034751208a",
    "description": "Extract all the branches from a tree",
    "inputs": [
      {
        "name": "Data",
        "nickname": "D",
        "access": "tree",
        "description": "Data to explode"
      }
    ],
    "outputs": [
      {
        "name": "Branch 0",
        "nickname": "-",
        "access": "tree",
        "description": "All data inside the branch at index: 0"
      },
      {
        "name": "Branch 1",
        "nickname": "-",
        "access": "tree",
        "description": "All data inside the branch at index: 1"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Text",
    "name": "Format",
    "nickname": "Format",
    "guid": "758d91a0-4aec-47f8-9671-16739a8a2c5d",
    "description": "Format some data using placeholders and formatting tags",
    "inputs": [
      {
        "name": "Format",
        "nickname": "F",
        "access": "item",
        "description": "Text format"
      },
      {
        "name": "Culture",
        "nickname": "C",
        "access": "item",
        "description": "Formatting culture"
      },
      {
        "name": "Data 0",
        "nickname": "0",
        "access": "item",
        "description": "Data to insert at {0} placeholders"
      },
      {
        "name": "Data 1",
        "nickname": "1",
        "access": "item",
        "description": "Data to insert at {1} placeholders"
      }
    ],
    "outputs": [
      {
        "name": "Text",
        "nickname": "T",
        "access": "item",
        "description": "Formatted text"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Tree",
    "name": "Clean Tree",
    "nickname": "Clean",
    "guid": "7991bc5f-8a01-4768-bfb0-a39357ac6b84",
    "description": "Removed all null and invalid items from a data tree.",
    "inputs": [
      {
        "name": "Tree",
        "nickname": "T",
        "access": "tree",
        "description": "Data tree to clean"
      },
      {
        "name": "Clean Invalid",
        "nickname": "X",
        "access": "item",
        "description": "Remove invalid items in addition to null items."
      },
      {
        "name": "Clean Empty",
        "nickname": "E",
        "access": "item",
        "description": "Remove empty branches."
      }
    ],
    "outputs": [
      {
        "name": "Tree",
        "nickname": "T",
        "access": "tree",
        "description": "Spotless data tree"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "List",
    "name": "Replace Items",
    "nickname": "Replace",
    "guid": "7a218bfb-b93d-4c1f-83d3-5a0b909dd60b",
    "description": "Replace certain items in a list.",
    "inputs": [
      {
        "name": "List",
        "nickname": "L",
        "access": "list",
        "description": "List to modify"
      },
      {
        "name": "Item",
        "nickname": "I",
        "access": "list",
        "description": "Items to replace with. If no items are supplied, nulls will be inserted."
      },
      {
        "name": "Indices",
        "nickname": "i",
        "access": "list",
        "description": "Replacement index for each item"
      },
      {
        "name": "Wrap",
        "nickname": "W",
        "access": "item",
        "description": "If true, indices will be wrapped"
      }
    ],
    "outputs": [
      {
        "name": "List",
        "nickname": "L",
        "access": "list",
        "description": "List with replaced values"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Sets",
    "name": "Disjoint",
    "nickname": "Disjoint",
    "guid": "81800098-1060-4e2b-80d4-17f835cc825f",
    "description": "Test whether two sets are disjoint.",
    "inputs": [
      {
        "name": "Set A",
        "nickname": "A",
        "access": "list",
        "description": "First set."
      },
      {
        "name": "Set B",
        "nickname": "B",
        "access": "list",
        "description": "Second set."
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "R",
        "access": "item",
        "description": "True if none of the items in A occur in B."
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Sets",
    "name": "Set Intersection",
    "nickname": "Intersection",
    "guid": "82f19c48-9e73-43a4-ae6c-3a8368099b08",
    "description": "Creates the intersection of two sets (the collection of unique objects present in both sets).",
    "inputs": [
      {
        "name": "Set A",
        "nickname": "A",
        "access": "list",
        "description": "Data for set Intersection"
      },
      {
        "name": "Set B",
        "nickname": "B",
        "access": "list",
        "description": "Data for set Intersection"
      }
    ],
    "outputs": [
      {
        "name": "Union",
        "nickname": "U",
        "access": "list",
        "description": "The Set Union of all input sets"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "List",
    "name": "Longest List",
    "nickname": "Long",
    "guid": "8440fd1b-b6e0-4bdb-aa93-4ec295c213e9",
    "description": "Grow a collection of lists to the longest length amongst them",
    "inputs": [
      {
        "name": "List (A)",
        "nickname": "A",
        "access": "list",
        "description": "List (A) to operate on"
      },
      {
        "name": "List (B)",
        "nickname": "B",
        "access": "list",
        "description": "List (B) to operate on"
      }
    ],
    "outputs": [
      {
        "name": "List (A)",
        "nickname": "A",
        "access": "list",
        "description": "Adjusted list (A)"
      },
      {
        "name": "List (B)",
        "nickname": "B",
        "access": "list",
        "description": "Adjusted list (B)"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Text",
    "name": "Characters",
    "nickname": "Chars",
    "guid": "86503240-d884-43f9-9323-efe30488a6e1",
    "description": "Break text into individual characters",
    "inputs": [
      {
        "name": "Text",
        "nickname": "T",
        "access": "item",
        "description": "Text to split."
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "C",
        "access": "list",
        "description": "Resulting characters"
      },
      {
        "name": "Unicode",
        "nickname": "U",
        "access": "list",
        "description": "Unicode value of character"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Tree",
    "name": "Merge",
    "nickname": "Merge",
    "guid": "86866576-6cc0-485a-9cd2-6f7d493f57f7",
    "description": "Merge two streams into one.",
    "inputs": [
      {
        "name": "Stream A",
        "nickname": "A",
        "access": "tree",
        "description": "Input stream #1"
      },
      {
        "name": "Stream B",
        "nickname": "B",
        "access": "tree",
        "description": "Input stream #2"
      }
    ],
    "outputs": [
      {
        "name": "Stream",
        "nickname": "S",
        "access": "item",
        "description": "Merged stream"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Tree",
    "name": "Graft Tree",
    "nickname": "Graft",
    "guid": "87e1d9ef-088b-4d30-9dda-8a7448a17329",
    "description": "Graft a data tree by adding an extra branch for every item.",
    "inputs": [
      {
        "name": "Tree",
        "nickname": "T",
        "access": "tree",
        "description": "Data tree to graft"
      }
    ],
    "outputs": [
      {
        "name": "Tree",
        "nickname": "T",
        "access": "tree",
        "description": "Grafted data tree"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Tree",
    "name": "Explode Tree",
    "nickname": "BANG!",
    "guid": "8a470a35-d673-4779-a65e-ba95765e59e4",
    "description": "Extract all the branches from a tree",
    "inputs": [
      {
        "name": "Tree",
        "nickname": "T",
        "access": "tree",
        "description": "Data tree to explode"
      }
    ],
    "outputs": [
      {
        "name": "Branch 0",
        "nickname": "0",
        "access": "tree",
        "description": "First branch in tree"
      },
      {
        "name": "Branch 1",
        "nickname": "1",
        "access": "tree",
        "description": "Second branch in tree"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Sets",
    "name": "Set Intersection",
    "nickname": "Intersection",
    "guid": "8a55f680-cf53-4634-a486-b828de92b71d",
    "description": "Creates the intersection of two sets (the collection of unique objects present in both sets).",
    "inputs": [
      {
        "name": "Set A",
        "nickname": "A",
        "access": "list",
        "description": "First set for Intersection"
      },
      {
        "name": "Set B",
        "nickname": "B",
        "access": "list",
        "description": "Second set for Intersection"
      }
    ],
    "outputs": [
      {
        "name": "Union",
        "nickname": "U",
        "access": "list",
        "description": "The Set Union of A and B"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Sets",
    "name": "Set Union",
    "nickname": "SUnion",
    "guid": "8eed5d78-7810-4ba1-968e-8a1f1db98e39",
    "description": "Creates the union of two sets (the collection of unique objects present in either set).",
    "inputs": [
      {
        "name": "Set A",
        "nickname": "A",
        "access": "list",
        "description": "Data for set Union."
      },
      {
        "name": "Set B",
        "nickname": "B",
        "access": "list",
        "description": "Data for set Union."
      }
    ],
    "outputs": [
      {
        "name": "Union",
        "nickname": "U",
        "access": "list",
        "description": "The Set Union of A and B."
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Sequence",
    "name": "Cull Nth",
    "nickname": "CullN",
    "guid": "932b9817-fcc6-4ac3-b5fd-c0e8eeadc53f",
    "description": "Cull (remove) every Nth element in a list.",
    "inputs": [
      {
        "name": "List",
        "nickname": "L",
        "access": "list",
        "description": "List to cull"
      },
      {
        "name": "Cull frequency",
        "nickname": "N",
        "access": "item",
        "description": "Cull frequency"
      }
    ],
    "outputs": [
      {
        "name": "List",
        "nickname": "L",
        "access": "list",
        "description": "Culled list"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Sequence",
    "name": "Range",
    "nickname": "Range",
    "guid": "9445ca40-cc73-4861-a455-146308676855",
    "description": "Create a range of numbers.",
    "inputs": [
      {
        "name": "Domain",
        "nickname": "D",
        "access": "item",
        "description": "Domain of numeric range"
      },
      {
        "name": "Steps",
        "nickname": "N",
        "access": "item",
        "description": "Number of steps"
      }
    ],
    "outputs": [
      {
        "name": "Range",
        "nickname": "R",
        "access": "list",
        "description": "Range of numbers"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Tree",
    "name": "Construct Path",
    "nickname": "Path",
    "guid": "946cb61e-18d2-45e3-8840-67b0efa26528",
    "description": "Construct a data tree branch path.",
    "inputs": [
      {
        "name": "Indices",
        "nickname": "I",
        "access": "list",
        "description": "Branch path indices"
      }
    ],
    "outputs": [
      {
        "name": "Branch",
        "nickname": "B",
        "access": "item",
        "description": "Branch path"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Sets",
    "name": "Create Set",
    "nickname": "CSet",
    "guid": "98c3c63a-e78a-43ea-a111-514fcf312c95",
    "description": "Creates the valid set from a list of items (a valid set only contains distinct elements).",
    "inputs": [
      {
        "name": "List",
        "nickname": "L",
        "access": "list",
        "description": "List of data."
      }
    ],
    "outputs": [
      {
        "name": "Set",
        "nickname": "S",
        "access": "list",
        "description": "A set of all the distincts values in L"
      },
      {
        "name": "Map",
        "nickname": "M",
        "access": "list",
        "description": "An index map from original indices to set indices"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Tree",
    "name": "Tree Statistics",
    "nickname": "TStat",
    "guid": "99bee19d-588c-41a0-b9b9-1d00fb03ea1a",
    "description": "Get some statistics regarding a data tree.",
    "inputs": [
      {
        "name": "Tree",
        "nickname": "T",
        "access": "tree",
        "description": "Data Tree to analyze"
      }
    ],
    "outputs": [
      {
        "name": "Paths",
        "nickname": "P",
        "access": "list",
        "description": "All the paths of the tree"
      },
      {
        "name": "Length",
        "nickname": "L",
        "access": "list",
        "description": "The length of each branch in the tree"
      },
      {
        "name": "Count",
        "nickname": "C",
        "access": "item",
        "description": "Number of paths and branches in the tree"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "List",
    "name": "Split List",
    "nickname": "Split",
    "guid": "9ab93e1a-ebdf-4090-9296-b000cff7b202",
    "description": "Split a list into separate parts.",
    "inputs": [
      {
        "name": "List",
        "nickname": "L",
        "access": "list",
        "description": "Base list"
      },
      {
        "name": "Index",
        "nickname": "i",
        "access": "item",
        "description": "Splitting index"
      }
    ],
    "outputs": [
      {
        "name": "List A",
        "nickname": "A",
        "access": "list",
        "description": "Items to the left of (i)"
      },
      {
        "name": "List B",
        "nickname": "B",
        "access": "list",
        "description": "Items to the right of and including (i)"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Sequence",
    "name": "RandomEx",
    "nickname": "RndEx",
    "guid": "a12dddbf-bb49-4ef4-aeb8-5653bc882cbd",
    "description": "Generate random data between extremes.",
    "inputs": [
      {
        "name": "Min",
        "nickname": "L0",
        "access": "item",
        "description": "Lower limit"
      },
      {
        "name": "Max",
        "nickname": "L1",
        "access": "item",
        "description": "Upper limit"
      },
      {
        "name": "Count",
        "nickname": "N",
        "access": "item",
        "description": "Number of values to generate"
      },
      {
        "name": "Seed",
        "nickname": "S",
        "access": "item",
        "description": "Random Seed"
      }
    ],
    "outputs": [
      {
        "name": "Values",
        "nickname": "V",
        "access": "list",
        "description": "Random values"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Tree",
    "name": "Flatten Tree",
    "nickname": "Flatten",
    "guid": "a13fcd5d-81af-4337-a32e-28dd7e23ae4c",
    "description": "Removes all branching information from a data tree.",
    "inputs": [
      {
        "name": "Data",
        "nickname": "D",
        "access": "tree",
        "description": "Data stream to flatten"
      }
    ],
    "outputs": [
      {
        "name": "Data",
        "nickname": "D",
        "access": "tree",
        "description": "Squished data"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Tree",
    "name": "Merge 08",
    "nickname": "M8",
    "guid": "a70aa477-0109-4e75-ba73-78725dca0274",
    "description": "Merge eight streams into one.",
    "inputs": [
      {
        "name": "Stream A",
        "nickname": "A",
        "access": "tree",
        "description": "Input stream #1"
      },
      {
        "name": "Stream B",
        "nickname": "B",
        "access": "tree",
        "description": "Input stream #2"
      },
      {
        "name": "Stream C",
        "nickname": "C",
        "access": "tree",
        "description": "Input stream #3"
      },
      {
        "name": "Stream D",
        "nickname": "D",
        "access": "tree",
        "description": "Input stream #4"
      },
      {
        "name": "Stream E",
        "nickname": "E",
        "access": "tree",
        "description": "Input stream #5"
      },
      {
        "name": "Stream F",
        "nickname": "F",
        "access": "tree",
        "description": "Input stream #6"
      },
      {
        "name": "Stream G",
        "nickname": "G",
        "access": "tree",
        "description": "Input stream #7"
      },
      {
        "name": "Stream H",
        "nickname": "H",
        "access": "tree",
        "description": "Input stream #8"
      }
    ],
    "outputs": [
      {
        "name": "Stream",
        "nickname": "S",
        "access": "item",
        "description": "Merged stream"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "List",
    "name": "Item Index",
    "nickname": "Index",
    "guid": "a759fd55-e6be-4673-8365-c28d5b52c6c0",
    "description": "Retrieve the index of a certain item in a list.",
    "inputs": [
      {
        "name": "List",
        "nickname": "L",
        "access": "list",
        "description": "List to search"
      },
      {
        "name": "Item",
        "nickname": "i",
        "access": "item",
        "description": "Item to search for"
      }
    ],
    "outputs": [
      {
        "name": "Index",
        "nickname": "i",
        "access": "item",
        "description": "The index of item in the list, or -1 if the item could not be found."
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Sets",
    "name": "Set Union",
    "nickname": "SUnion",
    "guid": "ab34845d-4ab9-4ff4-8870-eedd0c5594cb",
    "description": "Creates the union of two sets (the collection of unique objects present in either set).",
    "inputs": [
      {
        "name": "Set A",
        "nickname": "A",
        "access": "list",
        "description": "First set for Union."
      },
      {
        "name": "Set B",
        "nickname": "B",
        "access": "list",
        "description": "Second set for Union."
      }
    ],
    "outputs": [
      {
        "name": "Union",
        "nickname": "U",
        "access": "list",
        "description": "The Set Union of A and B."
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Tree",
    "name": "Merge 06",
    "nickname": "M6",
    "guid": "ac9b4faf-c9d5-4f6a-a5e9-58c0c2cac116",
    "description": "Merge six streams into one.",
    "inputs": [
      {
        "name": "Stream A",
        "nickname": "A",
        "access": "tree",
        "description": "Input stream #1"
      },
      {
        "name": "Stream B",
        "nickname": "B",
        "access": "tree",
        "description": "Input stream #2"
      },
      {
        "name": "Stream C",
        "nickname": "C",
        "access": "tree",
        "description": "Input stream #3"
      },
      {
        "name": "Stream D",
        "nickname": "D",
        "access": "tree",
        "description": "Input stream #4"
      },
      {
        "name": "Stream E",
        "nickname": "E",
        "access": "tree",
        "description": "Input stream #5"
      },
      {
        "name": "Stream F",
        "nickname": "F",
        "access": "tree",
        "description": "Input stream #6"
      }
    ],
    "outputs": [
      {
        "name": "Stream",
        "nickname": "S",
        "access": "item",
        "description": "Merged stream"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Text",
    "name": "Text Case",
    "nickname": "Case",
    "guid": "b1991128-8bf1-4dea-8497-4b7188a64e9d",
    "description": "Change the CaSiNg of a piece of text",
    "inputs": [
      {
        "name": "Text",
        "nickname": "T",
        "access": "item",
        "description": "Text to modify"
      },
      {
        "name": "Culture",
        "nickname": "C",
        "access": "item",
        "description": "Cultural rules for text casing"
      }
    ],
    "outputs": [
      {
        "name": "Upper Case",
        "nickname": "U",
        "access": "item",
        "description": "Upper case representation of T"
      },
      {
        "name": "Lower Case",
        "nickname": "L",
        "access": "item",
        "description": "Lower case representation of T"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "List",
    "name": "Sub List",
    "nickname": "SubSet",
    "guid": "b333ff42-93bd-406b-8e17-15780719b6ec",
    "description": "Extract a subset from a list.",
    "inputs": [
      {
        "name": "List",
        "nickname": "L",
        "access": "list",
        "description": "Base list"
      },
      {
        "name": "Domain",
        "nickname": "D",
        "access": "item",
        "description": "Domain of indices to copy"
      },
      {
        "name": "Wrap",
        "nickname": "W",
        "access": "item",
        "description": "Remap indices that overshoot list domain"
      }
    ],
    "outputs": [
      {
        "name": "List",
        "nickname": "L",
        "access": "list",
        "description": "Subset of base list"
      },
      {
        "name": "Index",
        "nickname": "I",
        "access": "list",
        "description": "Indices of subset items"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Sets",
    "name": "Find similar member",
    "nickname": "FSim",
    "guid": "b4d4235f-14ff-4d4e-a29a-b358dcd2baf4",
    "description": "Find the most similar member in a set.",
    "inputs": [
      {
        "name": "Data",
        "nickname": "D",
        "access": "item",
        "description": "Data to search for."
      },
      {
        "name": "Set",
        "nickname": "S",
        "access": "list",
        "description": "Set to search."
      }
    ],
    "outputs": [
      {
        "name": "Hit",
        "nickname": "H",
        "access": "item",
        "description": "Member in S closest to D."
      },
      {
        "name": "Index",
        "nickname": "i",
        "access": "item",
        "description": "Index of H in set."
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Tree",
    "name": "Merge 04",
    "nickname": "M4",
    "guid": "b5be5d1f-717f-493c-b958-816957f271fd",
    "description": "Merge four streams into one.",
    "inputs": [
      {
        "name": "Stream A",
        "nickname": "A",
        "access": "tree",
        "description": "Input stream #1"
      },
      {
        "name": "Stream B",
        "nickname": "B",
        "access": "tree",
        "description": "Input stream #2"
      },
      {
        "name": "Stream C",
        "nickname": "C",
        "access": "tree",
        "description": "Input stream #3"
      },
      {
        "name": "Stream D",
        "nickname": "D",
        "access": "tree",
        "description": "Input stream #4"
      }
    ],
    "outputs": [
      {
        "name": "Stream",
        "nickname": "S",
        "access": "item",
        "description": "Merged stream"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Sequence",
    "name": "Random",
    "nickname": "Random",
    "guid": "b7e4e0ef-a01d-48c4-93be-2a12d4417e22",
    "description": "Generate a list of pseudo random numbers.",
    "inputs": [
      {
        "name": "Range",
        "nickname": "R",
        "access": "item",
        "description": "Domain of random numeric range"
      },
      {
        "name": "Number",
        "nickname": "N",
        "access": "item",
        "description": "Number of random values"
      },
      {
        "name": "Seed",
        "nickname": "S",
        "access": "item",
        "description": "Seed of random engine"
      },
      {
        "name": "Integers",
        "nickname": "I",
        "access": "item",
        "description": "Limit to integers only"
      }
    ],
    "outputs": [
      {
        "name": "Range",
        "nickname": "R",
        "access": "list",
        "description": "Range of random numbers"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Tree",
    "name": "Unflatten Tree",
    "nickname": "Unflatten",
    "guid": "b8e2aa8f-8830-4ee1-bb59-613ea279c281",
    "description": "Unflatten a data tree by moving items back into branches.",
    "inputs": [
      {
        "name": "Tree",
        "nickname": "T",
        "access": "tree",
        "description": "Data tree to unflatten"
      },
      {
        "name": "Guide",
        "nickname": "G",
        "access": "tree",
        "description": "Guide data tree that defines the path layout"
      }
    ],
    "outputs": [
      {
        "name": "Tree",
        "nickname": "T",
        "access": "tree",
        "description": "Unflattened data tree"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Sets",
    "name": "Replace Members",
    "nickname": "Replace",
    "guid": "bafac914-ede4-4a59-a7b2-cc41bc3de961",
    "description": "Replace members in a set.",
    "inputs": [
      {
        "name": "Set",
        "nickname": "S",
        "access": "list",
        "description": "Set to operate on."
      },
      {
        "name": "Find",
        "nickname": "F",
        "access": "list",
        "description": "Item(s) to replace."
      },
      {
        "name": "Replace",
        "nickname": "R",
        "access": "list",
        "description": "Item(s) to replace with."
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "R",
        "access": "list",
        "description": "Sets with replaced members."
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Text",
    "name": "Text Case",
    "nickname": "Case",
    "guid": "bdd2a14a-1302-4152-a484-7198716d1a11",
    "description": "Change the CaSiNg of a piece of text",
    "inputs": [
      {
        "name": "Text",
        "nickname": "T",
        "access": "item",
        "description": "Text to operate on."
      }
    ],
    "outputs": [
      {
        "name": "Upper Case",
        "nickname": "U",
        "access": "item",
        "description": "Upper case representation of S"
      },
      {
        "name": "Lower Case",
        "nickname": "L",
        "access": "item",
        "description": "Lower case representation of S"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Tree",
    "name": "Replace Paths",
    "nickname": "Replace",
    "guid": "bfaaf799-77dc-4f31-9ad8-2f7d1a80aeb0",
    "description": "Find & replace paths in a data tree",
    "inputs": [
      {
        "name": "Data",
        "nickname": "D",
        "access": "tree",
        "description": "Data stream to process"
      },
      {
        "name": "Search",
        "nickname": "S",
        "access": "list",
        "description": "Search masks"
      },
      {
        "name": "Replace",
        "nickname": "R",
        "access": "list",
        "description": "Respective replacement paths"
      }
    ],
    "outputs": [
      {
        "name": "Data",
        "nickname": "D",
        "access": "tree",
        "description": "Processed tree data"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Tree",
    "name": "Tree Item",
    "nickname": "Item",
    "guid": "c1ec65a3-bda4-4fad-87d0-edf86ed9d81c",
    "description": "Retrieve a specific item from a data tree.",
    "inputs": [
      {
        "name": "Tree",
        "nickname": "T",
        "access": "tree",
        "description": "Data Tree"
      },
      {
        "name": "Path",
        "nickname": "P",
        "access": "item",
        "description": "Data tree branch path"
      },
      {
        "name": "Index",
        "nickname": "i",
        "access": "item",
        "description": "Item index"
      },
      {
        "name": "Wrap",
        "nickname": "W",
        "access": "item",
        "description": "Wrap index to list bounds"
      }
    ],
    "outputs": [
      {
        "name": "Element",
        "nickname": "E",
        "access": "item",
        "description": "Item at {P:i'}"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Sequence",
    "name": "Repeat Data",
    "nickname": "Repeat",
    "guid": "c40dc145-9e36-4a69-ac1a-6d825c654993",
    "description": "Repeat a pattern until it reaches a certain length.",
    "inputs": [
      {
        "name": "Data",
        "nickname": "D",
        "access": "list",
        "description": "Pattern to repeat"
      },
      {
        "name": "Length",
        "nickname": "L",
        "access": "item",
        "description": "Length of final pattern"
      }
    ],
    "outputs": [
      {
        "name": "Data",
        "nickname": "D",
        "access": "list",
        "description": "Repeated data"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "List",
    "name": "Null Item",
    "nickname": "Null",
    "guid": "c74efd0e-7fe3-4c2d-8c9d-295c5672fb13",
    "description": "Test a data item for null or invalidity",
    "inputs": [
      {
        "name": "Item",
        "nickname": "I",
        "access": "item",
        "description": "Item to test"
      }
    ],
    "outputs": [
      {
        "name": "Null Flags",
        "nickname": "N",
        "access": "item",
        "description": "True if item is Null"
      },
      {
        "name": "Invalid Flags",
        "nickname": "X",
        "access": "item",
        "description": "True if item is Invalid"
      },
      {
        "name": "Description",
        "nickname": "D",
        "access": "item",
        "description": "A textual description of the object state"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Text",
    "name": "Format",
    "nickname": "Format",
    "guid": "c8203c3c-6bcd-4f8c-a906-befd92ebf0cb",
    "description": "Format some text using placeholders and formatting tags",
    "inputs": [
      {
        "name": "Format",
        "nickname": "F",
        "access": "item",
        "description": "Text format"
      },
      {
        "name": "Data 0",
        "nickname": "0",
        "access": "item",
        "description": "Data to insert at {0} tags"
      },
      {
        "name": "Data 1",
        "nickname": "1",
        "access": "item",
        "description": "Data to insert at {1} tags"
      }
    ],
    "outputs": [
      {
        "name": "Text",
        "nickname": "T",
        "access": "item",
        "description": "Formatted text"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Tree",
    "name": "Entwine",
    "nickname": "Entwine",
    "guid": "c9785b8e-2f30-4f90-8ee3-cca710f82402",
    "description": "Flatten and combine a collection of data streams",
    "inputs": [
      {
        "name": "Branch {0;0}",
        "nickname": "{0;0}",
        "access": "tree",
        "description": "Data to entwine"
      },
      {
        "name": "Branch {0;1}",
        "nickname": "{0;1}",
        "access": "tree",
        "description": "Data to entwine"
      },
      {
        "name": "Branch {0;2}",
        "nickname": "{0;2}",
        "access": "tree",
        "description": "Data to entwine"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "R",
        "access": "item",
        "description": "Entwined result"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "List",
    "name": "Sort List",
    "nickname": "Sort",
    "guid": "cacb2c64-61b5-46db-825d-c61d5d09cc08",
    "description": "Sort a list of numeric keys.",
    "inputs": [
      {
        "name": "Keys",
        "nickname": "K",
        "access": "list",
        "description": "List of sortable keys"
      },
      {
        "name": "Values A",
        "nickname": "A",
        "access": "list",
        "description": "Optional list of values to sort synchronously"
      }
    ],
    "outputs": [
      {
        "name": "List",
        "nickname": "L",
        "access": "list",
        "description": "Sorted keys"
      },
      {
        "name": "Values A",
        "nickname": "A",
        "access": "list",
        "description": "Synchronous values in A"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Text",
    "name": "Sort Text",
    "nickname": "TSort",
    "guid": "cec16c67-7b8b-41f7-a5a5-f675177e524b",
    "description": "Sort a collection of text fragments",
    "inputs": [
      {
        "name": "Keys",
        "nickname": "K",
        "access": "list",
        "description": "Text fragments to sort (sorting key)"
      },
      {
        "name": "Values",
        "nickname": "V",
        "access": "list",
        "description": "Optional values to sort synchronously"
      },
      {
        "name": "Culture",
        "nickname": "C",
        "access": "item",
        "description": "Cultural sorting rules"
      }
    ],
    "outputs": [
      {
        "name": "Keys",
        "nickname": "K",
        "access": "list",
        "description": "Sorted text fragments"
      },
      {
        "name": "Values",
        "nickname": "V",
        "access": "list",
        "description": "Sorted values"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Sets",
    "name": "Set Difference (S)",
    "nickname": "ExDiff",
    "guid": "d2461702-3164-4894-8c10-ed1fc4b52965",
    "description": "Create the symmetric difference of two sets (the collection of objects present in A or B but not both).",
    "inputs": [
      {
        "name": "Set A",
        "nickname": "A",
        "access": "list",
        "description": "First set for symmetric difference."
      },
      {
        "name": "Set B",
        "nickname": "B",
        "access": "list",
        "description": "Second set for symmetric difference."
      }
    ],
    "outputs": [
      {
        "name": "ExDifference",
        "nickname": "X",
        "access": "list",
        "description": "The symmetric difference between A and B."
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Sets",
    "name": "Set Majority",
    "nickname": "Majority",
    "guid": "d4136a7b-7422-4660-9404-640474bd2725",
    "description": "Determine majority member presence amongst three sets.",
    "inputs": [
      {
        "name": "Set A",
        "nickname": "A",
        "access": "list",
        "description": "First set."
      },
      {
        "name": "Set B",
        "nickname": "B",
        "access": "list",
        "description": "Second set."
      },
      {
        "name": "Set C",
        "nickname": "C",
        "access": "list",
        "description": "Third set."
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "R",
        "access": "list",
        "description": "Set containing all unique elements in that occur in at least two of the input sets."
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Tree",
    "name": "Stream Gate",
    "nickname": "Gate",
    "guid": "d6313940-216b-487f-b511-6c8a5b87eae7",
    "description": "Redirects a stream into specific outputs.",
    "inputs": [
      {
        "name": "Stream",
        "nickname": "S",
        "access": "tree",
        "description": "Input stream"
      },
      {
        "name": "Gate",
        "nickname": "G",
        "access": "item",
        "description": "Gate index of output stream"
      }
    ],
    "outputs": [
      {
        "name": "Target 0",
        "nickname": "0",
        "access": "item",
        "description": "Output for Gate index 0"
      },
      {
        "name": "Target 1",
        "nickname": "1",
        "access": "item",
        "description": "Output for Gate index 1"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "List",
    "name": "Dispatch",
    "nickname": "Dispatch",
    "guid": "d8332545-21b2-4716-96e3-8559a9876e17",
    "description": "Dispatch the items in a list into two target lists.",
    "inputs": [
      {
        "name": "List",
        "nickname": "L",
        "access": "list",
        "description": "List to filter"
      },
      {
        "name": "Dispatch pattern",
        "nickname": "P",
        "access": "list",
        "description": "Dispatch pattern"
      }
    ],
    "outputs": [
      {
        "name": "List A",
        "nickname": "A",
        "access": "list",
        "description": "Dispatch target for True values"
      },
      {
        "name": "List B",
        "nickname": "B",
        "access": "list",
        "description": "Dispatch target for False values"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Tree",
    "name": "Split Tree",
    "nickname": "Split",
    "guid": "d8b1e7ac-cd31-4748-b262-e07e53068afc",
    "description": "Split a data tree into two parts using path masks.",
    "inputs": [
      {
        "name": "Data",
        "nickname": "D",
        "access": "tree",
        "description": "Tree to split"
      },
      {
        "name": "Masks",
        "nickname": "M",
        "access": "list",
        "description": "Splitting masks"
      }
    ],
    "outputs": [
      {
        "name": "Positive",
        "nickname": "P",
        "access": "tree",
        "description": "Positive set of data (all branches that match any of the masks)"
      },
      {
        "name": "Negative",
        "nickname": "N",
        "access": "tree",
        "description": "Negative set of data (all branches that do not match any of the masks"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Text",
    "name": "Text Length",
    "nickname": "Len",
    "guid": "dca05f6f-e3d9-42e3-b3bb-eb20363fb335",
    "description": "Get the length (character count) of some text",
    "inputs": [
      {
        "name": "Text",
        "nickname": "T",
        "access": "item",
        "description": "Text to measure."
      }
    ],
    "outputs": [
      {
        "name": "Length",
        "nickname": "L",
        "access": "item",
        "description": "Number of characters"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Sequence",
    "name": "Duplicate Data",
    "nickname": "Dup",
    "guid": "dd8134c0-109b-4012-92be-51d843edfff7",
    "description": "Duplicate data a predefined number of times.",
    "inputs": [
      {
        "name": "Data",
        "nickname": "D",
        "access": "list",
        "description": "Data to duplicate"
      },
      {
        "name": "Number",
        "nickname": "N",
        "access": "item",
        "description": "Number of duplicates"
      },
      {
        "name": "Order",
        "nickname": "O",
        "access": "item",
        "description": "Retain list order"
      }
    ],
    "outputs": [
      {
        "name": "Data",
        "nickname": "D",
        "access": "list",
        "description": "Duplicated data"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Sets",
    "name": "Carthesian Product",
    "nickname": "CProd",
    "guid": "deffaf1e-270a-4c15-a693-9216b68afd4a",
    "description": "Create the Carthesian product for two sets of identical cardinality.",
    "inputs": [
      {
        "name": "Set A",
        "nickname": "A",
        "access": "list",
        "description": "First set for carthesian product."
      },
      {
        "name": "Set B",
        "nickname": "B",
        "access": "list",
        "description": "Second set for carthesian product."
      }
    ],
    "outputs": [
      {
        "name": "Product",
        "nickname": "P",
        "access": "tree",
        "description": "Carthesian product of A and B."
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Tree",
    "name": "Deconstruct Path",
    "nickname": "DPath",
    "guid": "df6d9197-9a6e-41a2-9c9d-d2221accb49e",
    "description": "Deconstruct a data tree path into individual integers.",
    "inputs": [
      {
        "name": "Branch",
        "nickname": "B",
        "access": "item",
        "description": "Branch path"
      }
    ],
    "outputs": [
      {
        "name": "Indices",
        "nickname": "I",
        "access": "list",
        "description": "Branch path indices"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "List",
    "name": "Insert Items",
    "nickname": "Ins",
    "guid": "e2039b07-d3f3-40f8-af88-d74fed238727",
    "description": "Insert a collection of items into a list.",
    "inputs": [
      {
        "name": "List",
        "nickname": "L",
        "access": "list",
        "description": "List to modify"
      },
      {
        "name": "Item",
        "nickname": "I",
        "access": "list",
        "description": "Items to insert. If no items are supplied, nulls will be inserted."
      },
      {
        "name": "Indices",
        "nickname": "i",
        "access": "list",
        "description": "Insertion index for each item"
      },
      {
        "name": "Wrap",
        "nickname": "W",
        "access": "item",
        "description": "If true, indices will be wrapped"
      }
    ],
    "outputs": [
      {
        "name": "List",
        "nickname": "L",
        "access": "list",
        "description": "List with inserted values"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Sets",
    "name": "Set Difference",
    "nickname": "Difference",
    "guid": "e3b1a10c-4d49-4140-b8e6-0b5732a26c31",
    "description": "Create the difference of two sets (the collection of objects present in A but not in B).",
    "inputs": [
      {
        "name": "Set A",
        "nickname": "A",
        "access": "list",
        "description": "Set to subtract from."
      },
      {
        "name": "Set B",
        "nickname": "B",
        "access": "list",
        "description": "Substraction set."
      }
    ],
    "outputs": [
      {
        "name": "Union",
        "nickname": "U",
        "access": "list",
        "description": "The Set Difference of A minus B"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Text",
    "name": "Text Trim",
    "nickname": "Trim",
    "guid": "e4cb7168-5e32-4c54-b425-5a31c6fd685a",
    "description": "Remove whitespace characters from the start and end of some text.",
    "inputs": [
      {
        "name": "Text",
        "nickname": "T",
        "access": "item",
        "description": "Text to split."
      },
      {
        "name": "Start",
        "nickname": "S",
        "access": "item",
        "description": "Trim whitespace at start."
      },
      {
        "name": "End",
        "nickname": "E",
        "access": "item",
        "description": "Trim whitespace at end."
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "R",
        "access": "item",
        "description": "Trimmed text."
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Sequence",
    "name": "Series",
    "nickname": "Series",
    "guid": "e64c5fb1-845c-4ab1-8911-5f338516ba67",
    "description": "Create a series of numbers.",
    "inputs": [
      {
        "name": "Start",
        "nickname": "S",
        "access": "item",
        "description": "First number in the series"
      },
      {
        "name": "Step",
        "nickname": "N",
        "access": "item",
        "description": "Step size for each successive number"
      },
      {
        "name": "Count",
        "nickname": "C",
        "access": "item",
        "description": "Number of values in the series"
      }
    ],
    "outputs": [
      {
        "name": "Series",
        "nickname": "S",
        "access": "list",
        "description": "Series of numbers"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Tree",
    "name": "Null Check",
    "nickname": "Null",
    "guid": "e6859d1e-2b3d-4704-93ea-32714acae176",
    "description": "Test all items in a data tree for null.",
    "inputs": [
      {
        "name": "Data",
        "nickname": "D",
        "access": "tree",
        "description": "Data tree to test"
      }
    ],
    "outputs": [
      {
        "name": "Null",
        "nickname": "N",
        "access": "item",
        "description": "True if corresponding item is null"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Sequence",
    "name": "Duplicate data [OBSOLETE]",
    "nickname": "Dup",
    "guid": "e6e344aa-f45b-43d5-a2d9-9cf8e8e608dc",
    "description": "Duplicates some data a number of times",
    "inputs": [
      {
        "name": "Data",
        "nickname": "D",
        "access": "list",
        "description": "The data to duplicate"
      },
      {
        "name": "Number",
        "nickname": "N",
        "access": "item",
        "description": "Number of times to duplicate the data"
      }
    ],
    "outputs": [
      {
        "name": "Data",
        "nickname": "D",
        "access": "item",
        "description": "The duplicated data"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "List",
    "name": "Combine Data",
    "nickname": "Combine",
    "guid": "e7c80ff6-0299-4303-be36-3080977c14a1",
    "description": "Combine non-null items out of several inputs",
    "inputs": [
      {
        "name": "Input 0",
        "nickname": "0",
        "access": "item",
        "description": "Data to combine"
      },
      {
        "name": "Input 1",
        "nickname": "1",
        "access": "item",
        "description": "Data to combine"
      }
    ],
    "outputs": [
      {
        "name": "Result",
        "nickname": "R",
        "access": "item",
        "description": "Resulting data with as few nulls as possible"
      },
      {
        "name": "Index",
        "nickname": "I",
        "access": "item",
        "description": "Index of input that was copied into result"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Sequence",
    "name": "Sequence",
    "nickname": "Seq",
    "guid": "e9b2d2a6-0377-4c1c-a89e-b3f219a95b4d",
    "description": "Generate a sequence of numbers",
    "inputs": [
      {
        "name": "Notation",
        "nickname": "N",
        "access": "item",
        "description": "Sequence notation"
      },
      {
        "name": "Length",
        "nickname": "L",
        "access": "item",
        "description": "Final length of sequence"
      },
      {
        "name": "Initial",
        "nickname": "I",
        "access": "list",
        "description": "Initial values in sequence"
      }
    ],
    "outputs": [
      {
        "name": "Sequence",
        "nickname": "S",
        "access": "list",
        "description": "Sequence"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Tree",
    "name": "Stream Filter",
    "nickname": "Filter",
    "guid": "eeafc956-268e-461d-8e73-ee05c6f72c01",
    "description": "Filters a collection of input streams",
    "inputs": [
      {
        "name": "Gate",
        "nickname": "G",
        "access": "item",
        "description": "Index of Gate stream"
      },
      {
        "name": "Stream 0",
        "nickname": "0",
        "access": "tree",
        "description": "Input stream at index 0"
      },
      {
        "name": "Stream 1",
        "nickname": "1",
        "access": "tree",
        "description": "Input stream at index 1"
      }
    ],
    "outputs": [
      {
        "name": "Stream",
        "nickname": "S",
        "access": "tree",
        "description": "Filtered stream"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Sequence",
    "name": "Jitter",
    "nickname": "Jitter",
    "guid": "f02a20f6-bb49-4e3d-b155-8ed5d3c6b000",
    "description": "Randomly shuffles a list of values.",
    "inputs": [
      {
        "name": "List",
        "nickname": "L",
        "access": "list",
        "description": "Values to shuffle"
      },
      {
        "name": "Jitter",
        "nickname": "J",
        "access": "item",
        "description": "Shuffling strength. (0.0 = no shuffling, 1.0 = complete shuffling)"
      },
      {
        "name": "Seed",
        "nickname": "S",
        "access": "item",
        "description": "Seed of shuffling engine"
      }
    ],
    "outputs": [
      {
        "name": "Values",
        "nickname": "V",
        "access": "list",
        "description": "Shuffled values"
      },
      {
        "name": "Indices",
        "nickname": "I",
        "access": "list",
        "description": "Index map of shuffled items"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "List",
    "name": "Replace Nulls",
    "nickname": "NullRep",
    "guid": "f3230ecb-3631-4d6f-86f2-ef4b2ed37f45",
    "description": "Replace nulls or invalid data with other data",
    "inputs": [
      {
        "name": "Items",
        "nickname": "I",
        "access": "list",
        "description": "Items to test for null"
      },
      {
        "name": "Replacements",
        "nickname": "R",
        "access": "list",
        "description": "Items to replace nulls with"
      }
    ],
    "outputs": [
      {
        "name": "Items",
        "nickname": "I",
        "access": "list",
        "description": "List without any nulls"
      },
      {
        "name": "Count",
        "nickname": "N",
        "access": "item",
        "description": "Number of items replaced"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Tree",
    "name": "Merge 05",
    "nickname": "M5",
    "guid": "f4b0f7b4-5a10-46c4-8191-58d7d66ffdff",
    "description": "Merge five streams into one.",
    "inputs": [
      {
        "name": "Stream A",
        "nickname": "A",
        "access": "tree",
        "description": "Input stream #1"
      },
      {
        "name": "Stream B",
        "nickname": "B",
        "access": "tree",
        "description": "Input stream #2"
      },
      {
        "name": "Stream C",
        "nickname": "C",
        "access": "tree",
        "description": "Input stream #3"
      },
      {
        "name": "Stream D",
        "nickname": "D",
        "access": "tree",
        "description": "Input stream #4"
      },
      {
        "name": "Stream E",
        "nickname": "E",
        "access": "tree",
        "description": "Input stream #5"
      }
    ],
    "outputs": [
      {
        "name": "Stream",
        "nickname": "S",
        "access": "item",
        "description": "Merged stream"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Text",
    "name": "Text Distance",
    "nickname": "TDist",
    "guid": "f7608c4d-836c-4adf-9d1f-3b04e6a2647d",
    "description": "Compute the Levenshtein distance between two fragments of text.",
    "inputs": [
      {
        "name": "Text A",
        "nickname": "A",
        "access": "item",
        "description": "First text fragment"
      },
      {
        "name": "Text B",
        "nickname": "B",
        "access": "item",
        "description": "Second text fragment"
      },
      {
        "name": "Case",
        "nickname": "C",
        "access": "item",
        "description": "Compare using case-sensitive matching"
      }
    ],
    "outputs": [
      {
        "name": "Distance",
        "nickname": "D",
        "access": "item",
        "description": "Levenshtein distance between the two fragments"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Tree",
    "name": "Flatten Tree",
    "nickname": "Flatten",
    "guid": "f80cfe18-9510-4b89-8301-8e58faf423bb",
    "description": "Flatten a data tree by removing all branching information.",
    "inputs": [
      {
        "name": "Tree",
        "nickname": "T",
        "access": "tree",
        "description": "Data tree to flatten"
      },
      {
        "name": "Path",
        "nickname": "P",
        "access": "item",
        "description": "Path of flattened tree"
      }
    ],
    "outputs": [
      {
        "name": "Tree",
        "nickname": "T",
        "access": "tree",
        "description": "Flattened data tree"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Tree",
    "name": "Relative Item",
    "nickname": "RelItem",
    "guid": "fac0d5be-e3ff-4bbb-9742-ec9a54900d41",
    "description": "Retrieve a relative item combo from a data tree",
    "inputs": [
      {
        "name": "Tree",
        "nickname": "T",
        "access": "tree",
        "description": "Tree to operate on"
      },
      {
        "name": "Offset",
        "nickname": "O",
        "access": "item",
        "description": "Relative offset for item combo"
      },
      {
        "name": "Wrap Paths",
        "nickname": "Wp",
        "access": "item",
        "description": "Wrap paths when the shift is out of bounds"
      },
      {
        "name": "Wrap Items",
        "nickname": "Wi",
        "access": "item",
        "description": "Wrap items when the shift is out of bounds"
      }
    ],
    "outputs": [
      {
        "name": "Item A",
        "nickname": "A",
        "access": "tree",
        "description": "Tree item"
      },
      {
        "name": "Item B",
        "nickname": "B",
        "access": "tree",
        "description": "Tree item relative to A"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Sequence",
    "name": "Split [OBSOLETE]",
    "nickname": "Split",
    "guid": "fbcf0d42-c9a5-4ca5-8d5b-567fb54abc43",
    "description": "This component is OBSOLETE. It has been replaced",
    "inputs": [
      {
        "name": "Data",
        "nickname": "D",
        "access": "list",
        "description": "Input data stream"
      },
      {
        "name": "Boolean",
        "nickname": "B",
        "access": "item",
        "description": "Boolean evaluation flag"
      }
    ],
    "outputs": [
      {
        "name": "False",
        "nickname": "F",
        "access": "item",
        "description": "Output stream in case of B = False"
      },
      {
        "name": "True",
        "nickname": "T",
        "access": "item",
        "description": "Output stream in case of B = True"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Tree",
    "name": "Prune Tree",
    "nickname": "Prune",
    "guid": "fe769f85-8900-45dd-ba11-ec9cd6c778c6",
    "description": "Remove small branches from a Data Tree.",
    "inputs": [
      {
        "name": "Tree",
        "nickname": "T",
        "access": "tree",
        "description": "Data tree to prune"
      },
      {
        "name": "Minimum",
        "nickname": "N0",
        "access": "item",
        "description": "Remove branches with fewer than N0 items."
      },
      {
        "name": "Maximum",
        "nickname": "N1",
        "access": "item",
        "description": "Remove branches with more than N1 items (use zero to ignore upper limit)."
      }
    ],
    "outputs": [
      {
        "name": "Tree",
        "nickname": "T",
        "access": "tree",
        "description": "Pruned tree"
      }
    ]
  },
  {
    "category": "Sets",
    "subcategory": "Sequence",
    "name": "Fibonacci",
    "nickname": "Fib",
    "guid": "fe99f302-3d0d-4389-8494-bd53f7935a02",
    "description": "Creates a Fibonacci sequence.",
    "inputs": [
      {
        "name": "Seed A",
        "nickname": "A",
        "access": "item",
        "description": "First seed number of the sequence"
      },
      {
        "name": "Seed B",
        "nickname": "B",
        "access": "item",
        "description": "Second seed number of the sequence"
      },
      {
        "name": "Number",
        "nickname": "N",
        "access": "item",
        "description": "Number of values in the sequence"
      }
    ],
    "outputs": [
      {
        "name": "Series",
        "nickname": "S",
        "access": "list",
        "description": "First N numbers in this Fibonacci sequence"
      }
    ]
  }
];
