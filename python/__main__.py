import pandas as pd

# import seaborn
import matplotlib.pyplot as plt
import matplotlib.animation as animation


# Mostly ripping from this link
# http://www.roboticslab.ca/wp-content/uploads/2012/11/robotics_lab_animation_example.txt
# Outputs: https://www.youtube.com/watch?v=j0GOzfVZaMc&ab_channel=NicolasOlmedo


def draw(data, filename):
    # Will need something to get data sets here
    # 06, 09    # This is the grid layout for this test
    # 10, 13
    disp06 = data["Displacement 6"]
    disp09 = data["Displacement 9"]
    disp10 = data["Displacement 10"]
    disp13 = data["Displacement 13"]

    # This is pretty inefficient and not at all automated right now, but I can fix that later
    # Get max y
    actualMaximum = max([max(disp06), max(disp09), max(disp10), max(disp13)])

    # Get min y
    actualMinimum = min([min(disp06), min(disp09), min(disp10), min(disp13)])
    plt.ylim(actualMinimum, actualMaximum)

    axsstuff = [(0, 0), (0, 1), (1, 0), (1, 1)]

    axs = []

    for axscoord in axsstuff:
        axs.append(plt.subplot2grid((2, 2), axscoord))

    # Set up figure and subplots
    figureName = (
        plt.figure()
    )  # https://matplotlib.org/stable/api/_as_gen/matplotlib.pyplot.figure.html?highlight=matplotlib%20pyplot%20figure#matplotlib.pyplot.figure
    figureName.suptitle("Sensor Displacement", fontsize=12)
    # https://matplotlib.org/stable/api/_as_gen/matplotlib.pyplot.subplot2grid.html?highlight=subplot2grid#matplotlib.pyplot.subplot2grid
    ax01 = plt.subplot2grid((2, 2), (0, 0))
    ax02 = plt.subplot2grid((2, 2), (0, 1))
    ax03 = plt.subplot2grid((2, 2), (1, 0))
    ax04 = plt.subplot2grid((2, 2), (1, 1))

    # Set axis label names
    # ---
    
    disps = [disp06, disp09, disp10, disp13]

    redDots = []
    for i, axs in enumerate(axs):
        redDots.append(axs.plot(0, disps[i][0]))

    [redDot1] = ax01.plot(0, [disp06[0]], "ro")
    [redDot2] = ax02.plot(0, [disp09[0]], "ro")

    def animate(i):
        #for dispi, redDot in enumerate(redDots):
        #    redDot.set_data(0, disps[dispi][i])
        redDot1.set_data(0, disp06[i])
        redDot2.set_data(0, disp09[i])
        return redDot1, redDot2
        #return redDots

    simulation = animation.FuncAnimation(
        figureName, animate, blit=True, frames=50, interval=20, repeat=False
    )
    simulation.save(filename=filename, fps=30, dpi=150)


if __name__ == "__main__":
    import time

    start = time.perf_counter()
    with open("input\Reformatted copy - 2014-06-12-17-27-42-gauges.csv") as f:
        data = pd.read_csv(f)
        draw(data, "output/sim.mp4")
    end = time.perf_counter()
    print(end - start)
