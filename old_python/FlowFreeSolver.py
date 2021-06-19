import cv2
import numpy as np
import copy
from Solver import solvePuzzle
import time
import imageio
import AStar

def main():
	#puzzles = ["RegularPack5x5_1","RegularPack5x5_2","RegularPack7x7_1","RegularPack9x9_30","13x13Mania_1","13x13Mania_49","15x15Mania_1","ExtremePack12x12_30","RectanglePack_1"]
	
	#puzzles = ["RegularPack5x5_1","RegularPack5x5_2","RegularPack7x7_1","RegularPack9x9_30"]
	puzzles = ["RegularPack5x5_1"]
	for puzzle in puzzles:
		start = time.time()
		run(puzzle)
		print(time.time() - start)
	#run()

def run(puzzle_name=None):
	puzzle = cv2.imread(puzzle_name+".jpg")
	grayPuzzle = cv2.cvtColor(puzzle, cv2.COLOR_BGR2GRAY)
	solvedPuzzleImage1 = copy.deepcopy(puzzle)
	solvedPuzzleImage2 = copy.deepcopy(puzzle)
	
	cv2.imwrite(puzzle_name+"_gray.jpg",grayPuzzle)

	res,thresh = cv2.threshold(grayPuzzle,30,255,cv2.THRESH_BINARY)
	cv2.imwrite(puzzle_name+"_thresh.jpg", thresh)
	
	dim = dimsFromGrid(thresh, puzzle)
	print(puzzle_name + ": " + str(dim))
	
	gridX = dim[0]
	gridY = dim[1]
	startX = dim[2]
	endX = dim[3]
	startY = dim[4]
	endY = dim[5]
	
	boxSize = (endY - startY)/gridY
	
	# Find circles
	circles = []
	for x in range(0, gridX):
		for y in range(0, gridY):
			dotX = int(startX+boxSize*x+boxSize/2)
			dotY = int(startY+boxSize*y+boxSize/2)
			
			px = puzzle[dotY,dotX]

			if px[0]>40 or px[1]>40 or px[2]>40:
				circle = [dotX, x, dotY, y, (int(px[0]),int(px[1]),int(px[2])), 0]
				cv2.circle(puzzle,(dotX,dotY),2,(40,0,200),3)
				circles.append(circle)
	cv2.imwrite(puzzle_name+"_boxdots.jpg", puzzle)
	
	grid = [[0 for i in range(gridX)] for j in range(gridY)]
	basicGrid = [[0 for i in range(gridX)] for j in range(gridY)]

	counter = 1
	counter2 = 0
	alpha = "-ABCDEFGHIJKLMNOPQRSTUVWXYZ"
	colorDict = {"-":(0,0,0)}
	for i in range(0, len(circles)-1):
		c1 = circles[i]

		if c1[5] == 0:
			c1[5] = 1
			counter2 += 1
			for j in range(i+1, len(circles)):
				c2 = circles[j]

				if c2[5] == 0:
					if compareColor(c1[4], c2[4]):
						c2[5] = 1
						counter2+=1

						grid[c1[3]][c1[1]] = (counter, c1[3], c1[1], c1[2], c1[0], c1[4])
						grid[c2[3]][c2[1]] = (counter, c2[3], c2[1], c2[2], c2[0], c2[4])
						basicGrid[c1[3]][c1[1]] = counter
						basicGrid[c2[3]][c2[1]] = counter
						
						colorDict[alpha[counter]] = c1[4]
						
						
						counter+=1
	
	solvedPuzzleSequence = solveGridMain(basicGrid, int(len(circles)/2))
	
	solvedPuzzleGrid = solvedPuzzleSequence[-1]
	for x in range(0, gridX):
		for y in range(0, gridY):
			dotX = int(startX+boxSize*x+boxSize/2)
			dotY = int(startY+boxSize*y+boxSize/2)
			
			color = colorDict[solvedPuzzleGrid[y][x][0:1].upper()]
			
			cv2.circle(solvedPuzzleImage1,(dotX,dotY),int(boxSize/4),color,3)
	cv2.imwrite(puzzle_name+"_solved.jpg", solvedPuzzleImage1[startY:endY, startX:endX])
	
	gif = False
	#gif = True
	images = []
	print(len(solvedPuzzleSequence))
	if gif:
		filenames = []
		with imageio.get_writer(puzzle_name+".gif", mode='I', subrectangles=True) as writer:
			if len(solvedPuzzleSequence) > 1000:
				r = range(len(solvedPuzzleSequence)-1000, len(solvedPuzzleSequence))
			else: 
				r = range(len(solvedPuzzleSequence))
			for i in r:
				solvedPuzzleGrid = solvedPuzzleSequence[i]
				for x in range(0, gridX):
					for y in range(0, gridY):
						dotX = int(startX+boxSize*x+boxSize/2)
						dotY = int(startY+boxSize*y+boxSize/2)
						
						color = colorDict[solvedPuzzleGrid[y][x][0:1].upper()]
						
						cv2.circle(solvedPuzzleImage2,(dotX,dotY),int(boxSize/4),color,3)
				cv2.imwrite(puzzle_name+"_solved_step.jpg", solvedPuzzleImage2[startY:endY, startX:endX],[int(cv2.IMWRITE_JPEG_QUALITY), 50])
				writer.append_data(imageio.imread(puzzle_name+"_solved_step.jpg"))
			
		
		#imageio.mimsave(puzzle_name+'.gif', images)

def solveGridMain(basicGrid, numFlows):
	alpha = "-ABCDEFGHIJKLMNOPQRSTUVWXYZ"
	aGrid = []
	for arr in basicGrid:
		arr2 = []
		for i in arr:
			cell = alpha[i] + '-'
			if i > 0:
				cell += '0'
				alpha = alpha[:i] + alpha[i].lower() + alpha[i+1:]
			else:
				cell += '-'
			arr2.append(cell)
		aGrid.append(arr2)
	alpha = "-ABCDEFGHIJKLMNOPQRSTUVWXYZ"
	#printGrid(aGrid, True)
	flowStatus = []
	for i in range(numFlows):
		# [letter, length, finished]
		flowStatus.append([alpha[i+1], 0, 0])
	
	
	sizeY = len(aGrid)
	sizeX = len(aGrid[0])
	
	solvedPuzzle = solvePuzzle(aGrid, sizeX, sizeY, numFlows)
	
	return solvedPuzzle
	
def printGrid(grid, clean=False):
	pGrid = copy.deepcopy(grid)
	if clean:
		for y in range(len(pGrid)):
			for x in range(len(pGrid[y])):
				pGrid[y][x] = pGrid[y][x][0]
	for arr in pGrid:
		print(arr)
	print("\n\n")
		
def compareColor(c1, c2):
	return (abs(c1[0] - c2[0])<10 and abs(c1[1] - c2[1])<10 and abs(c1[2] - c2[2])<10)
	
def dimsFromGrid(thresh, puzzle):
	minLineLength= 100
	maxLineGap = 10
	lines = cv2.HoughLinesP(thresh,1,np.pi/180,500,minLineLength,maxLineGap)
	gridLinesX = []
	gridLinesY = []

	for line in lines:
		line = line[0]
		x1 = line[0]
		y1 = line[1]
		x2 = line[2]
		y2 = line[3]
		
		cv2.line(puzzle,(x1,y1),(x2,y2),(0,255,0),2)
		dx = abs(x1-x2)
		dy = abs(y1-y2)
		if dx<=10 or dy<=10:
			if dx == 0:
				gridLinesX.append(x1)
			else:
				gridLinesY.append(y1)
	cv2.imwrite('lines.jpg', puzzle)
	
	dim = (dimFromGridLines(gridLinesX)-1,dimFromGridLines(gridLinesY)-1,min(gridLinesX),max(gridLinesX),min(gridLinesY),max(gridLinesY))
	return dim
	
def dimFromGridLines(gridLines):
	hist = np.histogram(gridLines,40)[0]
	
	flag = 0
	count = 0
	for num in hist:
		if flag==0 and num>0:
			count+=1
			flag = 1
		if num==0:
			flag = 0
	return count

if __name__ == "__main__":
	main()