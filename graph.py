#!/bin/python3

import json
import matplotlib.pyplot as plt 

pressures = json.loads(open("./pressures.json").read())
print(pressures)

g_loads = json.loads(open("./g_load.json").read())
print(g_loads)
    
g_x = []
for i in range(0, len(g_loads)):
    g_x.append(len(g_x))

p_x = []
for i in range(0, len(pressures)):
    p_x.append(len(p_x))

# plotting the points  
plt.plot(g_x, g_loads)
    
# naming the x axis 
plt.xlabel('sample') 
# naming the y axis 
plt.ylabel('acc (g)') 
    
# giving a title to my graph 
plt.title('Acceleration') 
    
# function to show the plot 
plt.show()

# plotting the points  
plt.plot(p_x, pressures)
    
# naming the x axis 
plt.xlabel('sample') 
# naming the y axis 
plt.ylabel('pressure (Pa)') 
    
# giving a title to my graph 
plt.title('Pressure') 
    
# function to show the plot 
plt.show()
