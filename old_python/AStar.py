# Adapted from https://www.redblobgames.com/pathfinding/a-star/implementation.html

import collections
import heapq
from random import shuffle

class Queue:
	def __init__(self):
		self.elements = collections.deque()
	
	def empty(self):
		return len(self.elements) == 0
	
	def put(self, x):
		self.elements.append(x)
	
	def get(self):
		return self.elements.popleft()

class SquareGrid:
	def __init__(self, width, height, puzzle):
		self.width = width
		self.height = height
		self.puzzle = puzzle
	
	def in_bounds(self, id):
		(x, y) = id
		return 0 <= x < self.width and 0 <= y < self.height
	
	def passable(self, id):
		(x, y) = id
		return self.puzzle[y][x][0] == "-"
	
	def neighbors(self, id):
		(x, y) = id
		results = [(x+1, y), (x, y-1), (x-1, y), (x, y+1)]
		if (x + y) % 2 == 0: results.reverse() # aesthetics
		results = filter(self.in_bounds, results)
		results = filter(self.passable, results)
		return results

def heuristic(a, b):
	(x1, y1) = a
	(x2, y2) = b
	return abs(x1 - x2) + abs(y1 - y2)

def aStar(puzzle, start, goal):
	(x1, y1) = start
	(x2, y2) = goal
	
	puzzle[y1][x1] = "---"
	puzzle[y2][x2] = "---"
	
	graph = SquareGrid(len(puzzle[0]), len(puzzle), puzzle)
	
	frontier = Queue()
	frontier.put(start)
	came_from = {}
	came_from[start] = None
	
	while not frontier.empty():
		current = frontier.get()
		
		if current == goal:
			break
		
		for next in graph.neighbors(current):
			if next not in came_from:
				frontier.put(next)
				came_from[next] = current
	
	return came_from
	
def isPossible(puzzle, start, goal):
	return goal in aStar(puzzle, start, goal)
	

		
def isValidCell(puzzle, x, y):
	return x>=0 and y>=0 and y<len(puzzle) and x<len(puzzle[0])