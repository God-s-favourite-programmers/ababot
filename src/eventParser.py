import re
import json
import requests
import datetime
from bs4 import BeautifulSoup


def listEvents():
    url = "https://abakus.no/events"
    listPage = requests.get(url).content
    soup = BeautifulSoup(listPage, "html.parser")
    events = []
    for eventCategory in soup.findAll("div", {"class": "EventList__eventGroup--1-Btpkldi0"}):
        anchors = eventCategory.findAll("a")
        events.extend([x["href"][-4:] for x in anchors])
    return events


def getEvent(eventId):
    url = "https://lego.abakus.no/api/v1/events/"+str(eventId)
    r = requests.get(url)
    data = json.loads(r.text)
    name = data["title"]
    description = data["description"]
    eventTime = data["startTime"]
    regex = re.search(r"(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})Z", eventTime)
    eventTime = datetime.datetime(int(regex.group(1)), int(regex.group(2)), int(regex.group(3)), int(regex.group(4)), int(regex.group(5)))
    eventLocation = data["location"]
    try:
        rawRegistrationOpen = data["pools"][0]["activationDate"]
        regex = re.search(r"(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})Z", rawRegistrationOpen)
        registrationOpen = datetime.datetime(int(regex.group(1)), int(regex.group(2)), int(regex.group(3)), int(regex.group(4)), int(regex.group(5)))
    except IndexError:
        registrationOpen = None
    return {
        "name": name,
        "description": description,
        "eventTime": eventTime,
        "eventLocation": eventLocation,
        "regsitrationOpen": registrationOpen,
        "url": "https://abakus.no/events/"+str(eventId)
    }


