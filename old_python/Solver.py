import copy
from functools import total_ordering
import AStar
import sys

ALPHA = "-ABCDEFGHIJKLMNOPQRSTUVWXYZ"
DIRECTIONS = [(-1,0),(1,0),(0,-1),(0,1)]

def solvePuzzle(puzzle, sizeX, sizeY, numFlows):
	flowStatuses = []
	for i in range(1, numFlows+1):
		id = ALPHA[i]
		start = findCell(puzzle, id, 0)
		end = findCell(puzzle, id.lower(), 0)
		status = FlowStatus(id, start, end)
		flowStatuses.append(status)
	
	global recurseCount
	global attemptedStates
	attemptedStates = []
	recurseCount = 0
	root = PuzzleState(puzzle, None, numFlows, flowStatuses)
	root.update()
	res = recursivelySovle(root)
	print("\n")
	print(recurseCount)
	return attemptedStates
#	return res
	
def recursivelySovle(puzzleState):
	global recurseCount, attemptedStates
	recurseCount+=1
	
	if puzzleState.hasIsolatedEmptyCell():
		return None
	
	attemptedStates.append(puzzleState.puzzle)
	
	if recurseCount % 100 == 0:
		'''print(".", end="")
		sys.stdout.flush()'''
		printGrid(puzzleState.puzzle, True)
		print("\n")
	
	#print("RS!")
	# Step 0: check if all flows are DONE
	if puzzleState.isDone():
		#print("isDone!")
		return [puzzleState.puzzle]
	# Step 1: create all children states
	count = 0
	
	#printGrid(puzzleState.puzzle, True)
	puzzleState.setFlowToMove()

	flowId = puzzleState.flowToMove
	if flowId == "-":
		return None
	#print("Moving: " + flowId)
	flowStatus = None
	for status in puzzleState.flowStatuses:
		if status.id == flowId:
			flowStatus = status
			break
	x = flowStatus.currentEndPoint[0]
	y = flowStatus.currentEndPoint[1]
	for dir in DIRECTIONS:
		newX = x+dir[0]
		newY = y+dir[1]
		if isValidCell(puzzleState.puzzle, newX, newY) and not isOccupied(puzzleState.puzzle, newX, newY):
			childPuzzle = copy.deepcopy(puzzleState.puzzle)
			childFlowStatuses = copy.deepcopy(puzzleState.flowStatuses)
			
			childPuzzle[newY][newX] = flowId + "-" + str(flowStatus.length+1)
			for status in childFlowStatuses:
				if status.id == flowId:
					status.length+=1
					status.currentEndPoint = (newX, newY)
					status.updateFlowStatus(childPuzzle)
					
			childPuzzleState = PuzzleState(childPuzzle, puzzleState, puzzleState.numFlows, childFlowStatuses)
			childPuzzleState.update()
			puzzleState.children[count] = childPuzzleState
		else:
			puzzleState.children[count] = None
		count+=1
	puzzleState.children = [x for x in puzzleState.children if x is not None]
	puzzleState.children.sort()
	# Step 2: recursively solve all children states
	for childPuzzleState in puzzleState.children:
		if not childPuzzleState == None:
			if childPuzzleState.hasImpossible():
				continue
			res = recursivelySovle(childPuzzleState)
			if not res == None:
				return [puzzleState.puzzle] + res
			else:
				attemptedStates.append(puzzleState.puzzle)
	return None

	
def numberOfPossibleMoves(puzzle, x, y):
	count = 0
	for dir in DIRECTIONS:
		newX = x+dir[0]
		newY = y+dir[1]
		if isValidCell(puzzle, newX, newY) and not isOccupied(puzzle, newX, newY):
			count+=1
	return count
	
def isOccupied(puzzle, x, y):
	contents = puzzle[y][x]
	return not (contents[0] == "-")
	
def isValidCell(puzzle, x, y):
	return x>=0 and y>=0 and y<len(puzzle) and x<len(puzzle[0])
	
def findCell(puzzle, letter, num):
	searchString = str(letter) + "-" + str(num)
	for y in range(len(puzzle)):
		col = puzzle[y]
		for x in range(len(col)):
			contents = col[x]
			if contents == searchString:
				return (x,y)
	
def printGrid(grid, clean=False):
	pGrid = copy.copy(grid)
	if clean:
		for y in range(len(pGrid)):
			for x in range(len(pGrid[y])):
				pGrid[y][x] = pGrid[y][x][0]
	for arr in pGrid:
		print(arr)
		
@total_ordering	
class PuzzleState(object):
	def __init__(self, puzzle, parent, numFlows, flowStatuses):
		self.numFlows = numFlows
		self.flowToMove = "-"
		self.parent = parent
		self.children = [None, None, None, None]
		self.puzzle = puzzle
		self.flowStatuses = flowStatuses
		self.totalFlowDistances = -1
		self.flowsComplete = 0
		
	def setFlowToMove(self):
		minPossible = 5000
		minPossibleID = []
		for status in self.flowStatuses:
			if not status.done:
				score = status.score(self.puzzle)
				if score < minPossible:
					minPossible = score
					minPossibleID = [status.id]
				if score == minPossible:
					minPossibleID.append(status.id)
		if minPossible == 0 or len(minPossibleID) == 0:
			self.flowToMove = "-"
		else:
			self.flowToMove = minPossibleID[0]
		'''minPossible = 5
		minPossibleID = []
		for status in self.flowStatuses:
			if not status.done:
				numPossible = numberOfPossibleMoves(self.puzzle, status.currentEndPoint[0], status.currentEndPoint[1])
				if numPossible < minPossible:
					minPossible = numPossible
					minPossibleID = [status.id]
				if numPossible == minPossible:
					minPossibleID.append(status.id)
				#print(status.id + ": " + str(numPossible))
		if minPossible == 0 or len(minPossibleID) == 0:
			self.flowToMove = "-"
		else:
			self.flowToMove = minPossibleID[0]'''
			
	def update(self):
		self.totalFlowDistances = 0
		self.flowsComplete = 0
		for status in self.flowStatuses:
			status.updateFlowStatus(self.puzzle)
			self.totalFlowDistances += status.distanceToEnd
			if status.done:
				self.flowsComplete += 1
	
	def isDone(self):
		# All flows must be completed
		for status in self.flowStatuses:
			if not status.done:
				return False
		# All cells must be filled
		for y in range(len(self.puzzle)):
			for x in range(len(self.puzzle[y])):
				if self.puzzle[y][x][0:1] == "-":
					return False
		return True
		
	def hasImpossible(self):
		for status in self.flowStatuses:
			status.updateFlowStatus(self.puzzle)
			if status.impossible:
				return True
		return False
		
	def hasIsolatedEmptyCell(self):
		for y in range(len(self.puzzle)):
			for x in range(len(self.puzzle[y])):
				if not isOccupied(self.puzzle, x, y):
					possible = False
					for flow in self.flowStatuses:
						if not flow.done:
							if AStar.isPossible(copy.deepcopy(self.puzzle), flow.currentEndPoint, (x,y)) and AStar.isPossible(copy.deepcopy(self.puzzle), (x,y), flow.endCoord):
								possible = True
					if not possible:
						return True
		return False
		
	def compare(self, other):
		if other == None:
			return -1
		score = 0
		
		for status in self.flowStatuses:
			status.updateFlowStatus(self.puzzle)
			if status.impossible:
				score += 100
				
		for status in other.flowStatuses:
			status.updateFlowStatus(other.puzzle)
			if status.impossible:
				score -= 100
				
		if self.hasIsolatedEmptyCell():
			score += 100
		if other.hasIsolatedEmptyCell():
			score -= 100

		if self.totalFlowDistances < other.totalFlowDistances:
			score -= 10
		else:
			score += 10
			
		if self.flowsComplete > other.flowsComplete:
			score -= 2
		else:
			score += 2	
			
		if score < 0:
			return -1
		if score == 0:
			return 0
		if score > 0:
			return 1
		
	def __lt__(self, other):
		return self.compare(other) == -1

	def __eq__(self, other):
		return False
		
def sqrt(x):
	return x**(.5)

@total_ordering
class FlowStatus(object):
	def __init__(self, id, start, end):
		self.id = id
		self.startCoord = start
		self.endCoord = end
		self.currentEndPoint = start
		self.length = 0
		self.done = False
		self.distanceToEnd = sqrt((start[0] - end[0])**2 + (start[1] - end[1])**2)
		self.impossible = False
		
	def updateFlowStatus(self, puzzle):
		self.impossible = False
		if numberOfPossibleMoves(puzzle, self.currentEndPoint[0], self.currentEndPoint[1]) == 0 or numberOfPossibleMoves(puzzle, self.endCoord[0], self.endCoord[1]) == 0:
			self.impossible = True
		if not AStar.isPossible(copy.deepcopy(puzzle), self.currentEndPoint, self.endCoord):

			self.impossible = True
		
		self.distanceToEnd = sqrt((self.currentEndPoint[0] - self.endCoord[0])**2 + (self.currentEndPoint[1] - self.endCoord[1])**2) #len(AStar.aStar(copy.deepcopy(puzzle), self.currentEndPoint, self.endCoord))#
		for dir in DIRECTIONS:
			newX = self.currentEndPoint[0]+dir[0]
			newY = self.currentEndPoint[1]+dir[1]
			if newX == self.endCoord[0] and newY == self.endCoord[1]:
				self.done = True
				self.impossible = False
				
	def score(self, puzzle):
		score = numberOfPossibleMoves(puzzle, self.currentEndPoint[0], self.currentEndPoint[1])
		return score
		
	def __lt__(self, other):
		if other == None:
			return True
		return self.score(other) < other.score

	def __eq__(self, other):
		if other == None:
			return True
		return self.score(other) == other.score