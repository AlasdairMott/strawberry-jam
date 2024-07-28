import bpy
import mathutils
import json
import os

DISTANCE_THRESHOLD = 6


class AABB:
    def __init__(self, center, half_extents):
        self.center = center
        self.half_extents = half_extents

    @staticmethod
    def from_bounds(min_corner, max_corner, swap_yz=False):
        center = (min_corner + max_corner) / 2
        half_extents = (max_corner - min_corner) / 2

        if swap_yz:
            center = mathutils.Vector((center.x, center.z, center.y))
            half_extents = mathutils.Vector(
                (half_extents.x, half_extents.z, half_extents.y)
            )

        return AABB(center, half_extents)

    def to_dict(self):
        return {"center": list(self.center), "half_extents": list(self.half_extents)}

    def __str__(self):
        return f"Center: {self.center}\n" f"Half Extents: {self.half_extents}"


def get_empties():
    """
    Returns a list of all empties in the current Blender scene.
    """
    return [obj for obj in bpy.context.scene.objects if obj.type == "EMPTY"]


def calculate_aabb(collection):
    """
    Calculates the axis-aligned bounding box (AABB) for a given collection.
    """
    min_corner = mathutils.Vector((float("inf"), float("inf"), float("inf")))
    max_corner = mathutils.Vector((float("-inf"), float("-inf"), float("-inf")))

    for obj in collection.objects:
        if obj.type == "MESH":
            for vertex in obj.bound_box:
                world_vertex = obj.matrix_world @ mathutils.Vector(vertex)
                min_corner = mathutils.Vector(
                    (min(min_corner[i], world_vertex[i]) for i in range(3))
                )
                max_corner = mathutils.Vector(
                    (max(max_corner[i], world_vertex[i]) for i in range(3))
                )

    return AABB.from_bounds(min_corner, max_corner, True)


def export_collections_as_glb(referenced_collections, output_dir):
    """
    Exports each referenced collection as a GLB file.
    """
    for collection in referenced_collections:
        filepath = os.path.join(output_dir, f"{collection.name}.glb")
        bpy.ops.object.select_all(action="DESELECT")
        for obj in collection.objects:
            obj.select_set(True)
        bpy.context.view_layer.objects.active = collection.objects[0]
        bpy.ops.export_scene.gltf(
            filepath=filepath, use_selection=True, export_format="GLB"
        )


def calculate_relative_transform(obj_a, obj_b):
    """
    Calculate the transform matrix that brings obj_a to obj_b.
    Adjust the matrix to be compatible with Y-up coordinate system.
    """
    # Get world matrices of the objects
    matrix_a = obj_a.matrix_world
    matrix_b = obj_b.matrix_world

    # Calculate the inverse of the matrix of obj_a
    matrix_a_inv = matrix_a.inverted()

    # Calculate the relative transform matrix
    relative_matrix = matrix_a_inv @ matrix_b

    # Convert the relative matrix to Y-up coordinate system
    relative_matrix_y_up = convert_to_y_up(relative_matrix)

    return relative_matrix_y_up


def convert_to_y_up(matrix):
    """
    Convert a transformation matrix from Z-up to Y-up coordinate system.
    """
    z_to_y_up = mathutils.Matrix(
        [[1, 0, 0, 0], [0, 0, 1, 0], [0, -1, 0, 0], [0, 0, 0, 1]]
    )

    converted_matrix = z_to_y_up @ matrix @ z_to_y_up.inverted()

    return converted_matrix


def format_matrix(matrix):
    """
    Format a 4x4 matrix to a list of 16 elements, stored in column major order.
    """
    return [[matrix[j][i] for j in range(4)] for i in range(4)]


def generate_rust_file(json_data, output_dir):
    """Generate a Rust file with public constants for the exported GLB files."""
    rust_file_content = "// Auto-generated file\n\n"
    rust_file_content += "pub const EXPORTED_FILES: &[&str] = &[\n"
    for collection_name in json_data.keys():
        rust_file_content += f'    "{collection_name}",\n'
    rust_file_content += "];\n"

    rust_file_path = os.path.join(output_dir, "exports.rs")
    with open(rust_file_path, "w") as rust_file:
        rust_file.write(rust_file_content)


def main():
    """
    Main function to find all empties, print their transforms and instanced collections,
    export data to JSON and collections to GLB.
    """
    # Create the output directory if it doesn't exist
    current_dir = os.path.dirname(bpy.data.filepath)
    output_dir = os.path.join(os.path.dirname(current_dir), "assets", "exports")

    if not os.path.exists(output_dir):
        os.makedirs(output_dir)

    json_data = {}
    referenced_collections = set()

    empties = get_empties()
    empty_data = {empty.name: empty for empty in empties}

    for empty in empties:
        if (
            empty.instance_type == "COLLECTION"
            and empty.instance_collection is not None
        ):
            collection = empty.instance_collection
            aabb = calculate_aabb(collection)
            if collection.name not in json_data:
                json_data[collection.name] = {
                    "file": f"{collection.name}.glb",
                    "aabb": aabb.to_dict(),
                    "transforms": [],
                }
            referenced_collections.add(collection)

    # Calculate relative transforms
    for name_a, empty_a in empty_data.items():
        for name_b, empty_b in empty_data.items():
            if name_a != name_b:
                distance = (empty_a.location - empty_b.location).length
                if distance <= DISTANCE_THRESHOLD:
                    collection_a = (
                        empty_a.instance_collection.name
                        if empty_a.instance_collection
                        else "None"
                    )
                    collection_b = (
                        empty_b.instance_collection.name
                        if empty_b.instance_collection
                        else "None"
                    )
                    if collection_a != "None" and collection_b != "None":
                        transform_a_to_b = calculate_relative_transform(
                            empty_a, empty_b
                        )

                        json_data[collection_a]["transforms"].append(
                            [
                                collection_b,
                                format_matrix(transform_a_to_b),
                            ]
                        )

    # Save JSON data to file
    with open(os.path.join(output_dir, "data.json"), "w") as json_file:
        json.dump(json_data, json_file, indent=4)

    # Export referenced collections as GLB
    export_collections_as_glb(referenced_collections, output_dir)

    # Generate Rust file with exported file names
    rust_dir = os.path.join(os.path.dirname(current_dir), "src", "world")
    generate_rust_file(json_data, rust_dir)


# Execute the main function
if __name__ == "__main__":
    main()
