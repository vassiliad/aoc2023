import z3
import os
import typing


class Shard(typing.NamedTuple):
    pos: typing.List[float]
    vel: typing.List[float]


def parse_text(text: str) -> typing.List[Shard]:
    def extract_vector(text: str) -> typing.List[float]:
        return [float(x.strip()) for x in text.split(",")]

    ret = []
    for line in text.splitlines():
        line = line.strip()
        if not line:
            continue

        pos, vel = line.split("@", 1)
        pos = extract_vector(pos)
        vel = extract_vector(vel)

        ret.append(Shard(pos=pos, vel=vel))

    return ret


def solve(shards: typing.List[Shard]) -> int:
    # VV: I wrote this while waiting for `cargo build` to finish :)
    solver = z3.Solver()

    r_pos_x = z3.Real("r_pos_x")
    r_pos_y = z3.Real("r_pos_y")
    r_pos_z = z3.Real("r_pos_z")

    r_vel_x = z3.Real("r_vel_x")
    r_vel_y = z3.Real("r_vel_y")
    r_vel_z = z3.Real("r_vel_z")

    for (idx, shard) in enumerate(shards[:3]):
        i_t = z3.Real(f"t{idx}")

        rock_pos_x_t = r_pos_x + r_vel_x * i_t
        rock_pos_y_t = r_pos_y + r_vel_y * i_t
        rock_pos_z_t = r_pos_z + r_vel_z * i_t

        shard_pos_x_t = shard.pos[0] + shard.vel[0] * i_t
        shard_pos_y_t = shard.pos[1] + shard.vel[1] * i_t
        shard_pos_z_t = shard.pos[2] + shard.vel[2] * i_t

        solver.add(i_t > 0)
        solver.add(rock_pos_x_t == shard_pos_x_t)
        solver.add(rock_pos_y_t == shard_pos_y_t)
        solver.add(rock_pos_z_t == shard_pos_z_t)

    solver.check()

    model = solver.model()

    return model.eval(r_pos_x + r_pos_y + r_pos_z)


def main():
    path = os.path.join(os.getcwd(), "input", "mine")
    contents = open(path, "r").read()
    shards = parse_text(contents)

    solution = solve(shards)

    print(solution)


if __name__ == "__main__":
    main()
