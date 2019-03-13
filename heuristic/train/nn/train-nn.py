from __future__ import absolute_import, division, print_function

import sys
import pathlib

import matplotlib.pyplot as plt
import pandas as pd
import seaborn as sns

import tensorflow as tf
from tensorflow import keras
from tensorflow.keras import layers

EPOCHS = 1000

# The patience parameter is the amount of epochs to check for improvement
EARLY_STOP = keras.callbacks.EarlyStopping(monitor='val_loss', patience=30)

class PrintDot(keras.callbacks.Callback):
  def on_epoch_end(self, epoch, logs):
    if epoch % 100 == 0: print('')
    print('.', end='')

def plot_history(history):
  hist = pd.DataFrame(history.history)
  hist['epoch'] = history.epoch
  plt.figure()
  plt.xlabel('Epoch')
  plt.ylabel('Mean Abs Error [cost]')
  plt.plot(hist['epoch'], hist['mean_absolute_error'],
           label='Train Error')
  plt.plot(hist['epoch'], hist['val_mean_absolute_error'],
           label = 'Val Error')
  plt.ylim([0,5])
  plt.legend()
  plt.figure()
  plt.xlabel('Epoch')
  plt.ylabel('Mean Square Error [$cost^2$]')
  plt.plot(hist['epoch'], hist['mean_squared_error'],
           label='Train Error')
  plt.plot(hist['epoch'], hist['val_mean_squared_error'],
           label = 'Val Error')
  plt.ylim([0,20])
  plt.legend()
  plt.show()

# we hard-code the values instead of using stats so that integration with
# predictor using the model is easier
scaling = pd.DataFrame(data={
    'min': [-10000, -10000, -10000, -2300, -2300, -2300,  -6.0,  -6.0,  -6.0,   -3.2,    -3.2,  -3.2],
    'max': [ 10000,  10000,  10000,  2300,  2300,  2300,   6.0,   6.0,   6.0,    3.2,     3.2,   3.2],
}, index=[     'x',    'y',    'z',  'vx',  'vy',  'vz', 'avx', 'avy', 'avz', 'roll', 'pitch', 'yaw'])

# scale to range [0, 1]
# TODO try polar coordinates. for velocity: https://math.stackexchange.com/questions/2444965/relationship-between-cartesian-velocity-and-polar-velocity
def scale(x):
  return (x - scaling['min']) / (scaling['max'] - scaling['min'])

def build_model():
  model = keras.Sequential([
    layers.Dense(128, activation=tf.nn.relu, input_shape=[len(train_dataset.keys())]),
    layers.Dense(128, activation=tf.nn.relu),

    # these extra layers seem to hurt more than they help!
    #layers.Dropout(0.01),
    #layers.Dense(64, activation=tf.nn.relu),

    # this doesn't work as well as a single 64-wide layer
    #layers.Dense(12, activation=tf.nn.relu, input_shape=[len(train_dataset.keys())]),
    #layers.Dense(12, activation=tf.nn.relu),
    #layers.Dense(12, activation=tf.nn.relu),
    #layers.Dense(12, activation=tf.nn.relu),
    #layers.Dense(12, activation=tf.nn.relu),

    layers.Dense(1)
  ])
  #optimizer = tf.keras.optimizers.RMSprop(0.001)
  optimizer = tf.train.AdamOptimizer(0.001)
  model.compile(loss='mean_squared_error',
                optimizer=optimizer,
                metrics=['mean_absolute_error', 'mean_squared_error'])
  return model


# should be the time.csv from generate-data's time binary
dataset_path = sys.argv[1]

column_names = ['cost', 'x', 'y', 'z', 'vx', 'vy', 'vz', 'avx', 'avy', 'avz', 'roll', 'pitch', 'yaw']
raw_dataset = pd.read_csv(dataset_path, names=column_names,
                      na_values = "", #comment='\t',
                      sep=",", skipinitialspace=True)


# visualize the data!
pos_plot = sns.pairplot(raw_dataset[["cost", "x", "y", "z"]], diag_kind="kde")
pos_plot.savefig("./pos.fig.png")
vel_plot = sns.pairplot(raw_dataset[["cost", "vx", "vy", "vz"]], diag_kind="kde")
vel_plot.savefig("./vel.fig.png")
avel_plot = sns.pairplot(raw_dataset[["cost", "avx", "avy", "avz"]], diag_kind="kde")
avel_plot.savefig("./avel.fig.png")
rot_plot = sns.pairplot(raw_dataset[["cost", "roll", "pitch", "yaw"]], diag_kind="kde")
rot_plot.savefig("./rot.fig.png")
pos_rot_plot = sns.pairplot(raw_dataset[["cost", "x", "y", "yaw"]], diag_kind="kde")
pos_rot_plot.savefig("./pos_rot.fig.png")

dataset = raw_dataset.copy()
dataset.tail()

# we don't have missing data
# dataset.isna().sum()
# dataset = dataset.dropna()

# split into training vs test datasets
train_dataset = dataset.sample(frac=0.95,random_state=0)
test_dataset = dataset.drop(train_dataset.index)

# using stats from full dataset
stats = raw_dataset.describe()
stats.pop("cost")
stats = stats.transpose()
stats

train_labels = train_dataset.pop('cost')
test_labels = test_dataset.pop('cost')

scaled_train_dataset = scale(train_dataset)
scaled_test_dataset = scale(test_dataset)

# build and train moddel
model = build_model()
model.summary()
history = model.fit(scaled_train_dataset, train_labels, epochs=EPOCHS,
                    validation_split = 0.2, verbose=0, callbacks=[EARLY_STOP, PrintDot()])
plot_history(history)

# check against test set
loss, mae, mse = model.evaluate(scaled_test_dataset, test_labels, verbose=0)
print("Testing set Mean Abs Error: {:5.2f} cost".format(mae))

# plot all test predictions
test_predictions = model.predict(scaled_test_dataset).flatten()
plt.scatter(test_labels, test_predictions)
plt.xlabel('True Values [cost]')
plt.ylabel('Predictions [cost]')
plt.axis('equal')
plt.axis('square')
plt.xlim([0,plt.xlim()[1]])
plt.ylim([0,plt.ylim()[1]])
plt.plot([-100, 100], [-100, 100])
plt.show()

# error distribution
error = test_predictions - test_labels
plt.hist(error, bins = 25)
plt.xlabel("Prediction Error [cost]")
plt.ylabel("Count")
plt.show()

model.save('./simple_throttle_cost_model.h5')
saved_model_path = tf.contrib.saved_model.save_keras_model(model, "./simple_throttle_cost_saved_model")

