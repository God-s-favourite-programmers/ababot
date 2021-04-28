import datetime
import re
import pytz
from src.cogs.abakus.event import event

local_timezone = pytz.timezone("Europe/Oslo")

def generate_message(event_object:event, template:str):
    if event_object == None:
        raise ValueError("Event object is None")
    with open("./src/cogs/abakus/templates/"+template, "r") as f:
        msg = f.read()
    time = datetime.datetime.strftime(event_object.get_registration_open(), '%Y-%m-%d %H:%M:%S')
    startTime = datetime.datetime.strftime(event_object.get_event_time(), '%Y-%m-%d %H:%M:%S')
    return (msg.format(
        eventName=event_object.get_name(),
        eventDescription=event_object.get_description(),
        signupTime=time,
        eventLocation=event_object.get_event_location(),
        startTime=startTime,
        url=event_object.get_url()
    ))


def get_event_properties(message, template) -> event:
    with open("./src/cogs/abakus/templates/"+template, "r") as f:
        pattern = f.read()
    messageSearch = re.search(pattern, message.content)
    if messageSearch:
        name = messageSearch.group(1)
        description = messageSearch.group(2)
        startTime = messageSearch.group(3)
        startTime = local_timezone.localize(datetime.datetime.strptime(startTime, '%Y-%m-%d %H:%M:%S'))
        location = messageSearch.group(4)
        signupTime = messageSearch.group(5)
        signupTime = local_timezone.localize(datetime.datetime.strptime(signupTime, '%Y-%m-%d %H:%M:%S'))
        url = messageSearch.group(6)
        event_options = {
            "name": name,
            "description": description,
            "registration_open": signupTime,
            "event_location": location,
            "event_time": startTime,
            "url": url
        }
        event_object = event(**event_options)
        return event_object
    else:
        return



async def get_dm_history(user):
    if user.dm_channel:
        pass
    else:
        await user.create_dm()

    history = await user.dm_channel.history(limit=123).flatten()
    history = [x.content for x in history]
    return history
