from algorand import Algorand
from helper import sleep_sec


algorand = Algorand()
input()
sleep_sec(5)
algorand.set_secret("wow")
algorand.deploy_atomic()
print(algorand.appid)
sleep_sec(5)
algorand.withdraw("wow")
